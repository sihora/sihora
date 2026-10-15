//! Fine-grained reactive runtime (Leptos/Solid style).
//!
//! Arena-owned signals with `Copy` ID handles. Dependency tracking via
//! thread-local current observer. Owner scopes for cleanup. Batching.
//!
//! # Example
//! ```
//! use sihora::core::reactive::*;
//! use std::cell::Cell;
//! use std::rc::Rc;
//!
//! reset();
//!
//! let count = create_signal(0);
//! let runs = Rc::new(Cell::new(0));
//! let r = runs.clone();
//! create_effect(move || {
//!     let _ = get(count);
//!     r.set(r.get() + 1);
//! });
//! assert_eq!(runs.get(), 1);
//! set(count, 5);
//! assert_eq!(runs.get(), 2);
//! ```

use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use slotmap::SlotMap;

slotmap::new_key_type! {
    /// Identifies a signal in the reactive runtime arena.
    pub struct SignalId;
    /// Identifies an effect in the reactive runtime arena.
    pub struct EffectId;
    /// Identifies an owner scope in the reactive runtime arena.
    pub struct ScopeId;
}

/// A reactive signal handle. `Copy` — users hold these, the runtime owns the data.
///
/// # Example
/// ```
/// # sihora::core::reactive::reset();
/// let count = sihora::core::reactive::create_signal(0);
/// assert_eq!(sihora::core::reactive::get(count), 0);
/// sihora::core::reactive::set(count, 42);
/// assert_eq!(sihora::core::reactive::get(count), 42);
/// ```
pub struct Signal<T: 'static> {
    id: SignalId,
    _marker: PhantomData<T>,
}

impl<T> Copy for Signal<T> {}
impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> std::fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signal").field("id", &self.id).finish()
    }
}

impl<T> PartialEq for Signal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for Signal<T> {}

impl<T> std::hash::Hash for Signal<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// ---------------------------------------------------------------------------
// Internal storage
// ---------------------------------------------------------------------------

struct SignalData {
    value: Box<dyn Any>,
    subscribers: Vec<EffectId>,
}

struct EffectData {
    func: Rc<dyn Fn()>,
    deps: Vec<SignalId>,
}

struct ScopeData {
    signals: Vec<SignalId>,
    effects: Vec<EffectId>,
    children: Vec<ScopeId>,
    parent: Option<ScopeId>,
}

struct Runtime {
    signals: SlotMap<SignalId, SignalData>,
    effects: SlotMap<EffectId, EffectData>,
    scopes: SlotMap<ScopeId, ScopeData>,
    current_effect: Option<EffectId>,
    current_scope: Option<ScopeId>,
    pending: Vec<EffectId>,
    batch_depth: u32,
    flushing: bool,
    dirty: bool,
}

impl Runtime {
    fn new() -> Self {
        Self {
            signals: SlotMap::with_key(),
            effects: SlotMap::with_key(),
            scopes: SlotMap::with_key(),
            current_effect: None,
            current_scope: None,
            pending: Vec::new(),
            batch_depth: 0,
            flushing: false,
            dirty: false,
        }
    }
}

thread_local! {
    static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::new());
}

// ---------------------------------------------------------------------------
// Signal API
// ---------------------------------------------------------------------------

/// Create a new signal with the given initial value.
///
/// Returns a `Copy` handle. The value is owned by the reactive runtime.
///
/// # Example
/// ```
/// # sihora::core::reactive::reset();
/// let name = sihora::core::reactive::create_signal("hello".to_string());
/// assert_eq!(sihora::core::reactive::get(name), "hello");
/// ```
pub fn create_signal<T: 'static>(value: T) -> Signal<T> {
    RUNTIME.with(|rt| {
        let mut rt = rt.borrow_mut();
        let id = rt.signals.insert(SignalData {
            value: Box::new(value),
            subscribers: Vec::new(),
        });
        if let Some(scope_id) = rt.current_scope {
            if let Some(scope) = rt.scopes.get_mut(scope_id) {
                scope.signals.push(id);
            }
        }
        Signal {
            id,
            _marker: PhantomData,
        }
    })
}

/// Read a signal's current value.
///
/// If called inside an effect, registers a dependency so the effect
/// re-runs when this signal changes.
///
/// # Example
/// ```
/// # sihora::core::reactive::reset();
/// let x = sihora::core::reactive::create_signal(10);
/// assert_eq!(sihora::core::reactive::get(x), 10);
/// ```
pub fn get<T: Clone + 'static>(signal: Signal<T>) -> T {
    RUNTIME.with(|rt| {
        let mut rt = rt.borrow_mut();
        if let Some(effect_id) = rt.current_effect {
            if let Some(sig) = rt.signals.get_mut(signal.id) {
                if !sig.subscribers.contains(&effect_id) {
                    sig.subscribers.push(effect_id);
                }
            }
            if let Some(eff) = rt.effects.get_mut(effect_id) {
                if !eff.deps.contains(&signal.id) {
                    eff.deps.push(signal.id);
                }
            }
        }
        // INVARIANT: Signal<T> is created with T and set only via Signal<T> (same type).
        rt.signals[signal.id]
            .value
            .downcast_ref::<T>()
            .expect("signal type invariant: Signal<T> always stores T")
            .clone()
    })
}

/// Read a signal without tracking the dependency.
///
/// Useful inside a memo to avoid circular dependency with the output signal.
pub fn get_untracked<T: Clone + 'static>(signal: Signal<T>) -> T {
    RUNTIME.with(|rt| {
        let rt = rt.borrow();
        // INVARIANT: Signal<T> always stores T.
        rt.signals[signal.id]
            .value
            .downcast_ref::<T>()
            .expect("signal type invariant: Signal<T> always stores T")
            .clone()
    })
}

/// Write a new value to a signal, notifying subscribers.
///
/// # Example
/// ```
/// # sihora::core::reactive::reset();
/// let x = sihora::core::reactive::create_signal(0);
/// sihora::core::reactive::set(x, 42);
/// assert_eq!(sihora::core::reactive::get(x), 42);
/// ```
pub fn set<T: 'static>(signal: Signal<T>, value: T) {
    RUNTIME.with(|rt| {
        let mut rt = rt.borrow_mut();
        rt.signals[signal.id].value = Box::new(value);
        rt.dirty = true;
        let subs: Vec<EffectId> = rt.signals[signal.id].subscribers.clone();
        #[cfg(debug_assertions)]
        if !subs.is_empty() {
            eprintln!(
                "[sihora::reactive] signal {:?} written, {} effect(s) notified",
                signal.id,
                subs.len()
            );
        }
        for eid in subs {
            if !rt.pending.contains(&eid) {
                rt.pending.push(eid);
            }
        }
    });
    maybe_flush();
}

/// Update a signal by applying a function to its current value.
///
/// # Example
/// ```
/// # sihora::core::reactive::reset();
/// let count = sihora::core::reactive::create_signal(0);
/// sihora::core::reactive::update(count, |c| c + 1);
/// assert_eq!(sihora::core::reactive::get(count), 1);
/// ```
pub fn update<T: Clone + 'static>(signal: Signal<T>, f: impl FnOnce(T) -> T) {
    let old = get(signal);
    set(signal, f(old));
}

// ---------------------------------------------------------------------------
// Effect / Memo API
// ---------------------------------------------------------------------------

/// Create a reactive effect that re-runs when its signal dependencies change.
///
/// The closure runs immediately once to establish dependencies, then
/// re-runs whenever any read signal is written to.
///
/// # Example
/// ```
/// # sihora::core::reactive::reset();
/// use std::cell::Cell;
/// use std::rc::Rc;
///
/// let count = sihora::core::reactive::create_signal(0);
/// let log = Rc::new(Cell::new(0));
/// let log2 = log.clone();
/// sihora::core::reactive::create_effect(move || {
///     log2.set(sihora::core::reactive::get(count));
/// });
/// assert_eq!(log.get(), 0);
/// sihora::core::reactive::set(count, 5);
/// assert_eq!(log.get(), 5);
/// ```
pub fn create_effect(f: impl Fn() + 'static) {
    let func: Rc<dyn Fn()> = Rc::new(f);
    let id = RUNTIME.with(|rt| {
        let mut rt = rt.borrow_mut();
        let id = rt.effects.insert(EffectData {
            func: func.clone(),
            deps: Vec::new(),
        });
        if let Some(scope_id) = rt.current_scope {
            if let Some(scope) = rt.scopes.get_mut(scope_id) {
                scope.effects.push(id);
            }
        }
        id
    });
    run_effect(id);
}

/// Create a derived signal that caches the result of a computation.
///
/// Implemented as a signal backed by an effect. Re-computes only when
/// dependencies change. Skips update if the new value equals the old.
///
/// # Example
/// ```
/// # sihora::core::reactive::reset();
/// let count = sihora::core::reactive::create_signal(2);
/// let doubled = sihora::core::reactive::create_memo(move || {
///     sihora::core::reactive::get(count) * 2
/// });
/// assert_eq!(sihora::core::reactive::get(doubled), 4);
/// sihora::core::reactive::set(count, 5);
/// assert_eq!(sihora::core::reactive::get(doubled), 10);
/// ```
pub fn create_memo<T: Clone + PartialEq + 'static>(f: impl Fn() -> T + 'static) -> Signal<T> {
    let sig = create_signal(f());
    create_effect(move || {
        let new_val = f();
        let old_val = get_untracked(sig);
        if new_val != old_val {
            set(sig, new_val);
        }
    });
    sig
}

// ---------------------------------------------------------------------------
// Batching
// ---------------------------------------------------------------------------

/// Group multiple signal writes so effects flush only once at the end.
///
/// # Example
/// ```
/// # sihora::core::reactive::reset();
/// use std::cell::Cell;
/// use std::rc::Rc;
///
/// let a = sihora::core::reactive::create_signal(0);
/// let b = sihora::core::reactive::create_signal(0);
/// let runs = Rc::new(Cell::new(0));
/// let r = runs.clone();
/// sihora::core::reactive::create_effect(move || {
///     let _ = sihora::core::reactive::get(a) + sihora::core::reactive::get(b);
///     r.set(r.get() + 1);
/// });
/// assert_eq!(runs.get(), 1);
/// sihora::core::reactive::batch(|| {
///     sihora::core::reactive::set(a, 1);
///     sihora::core::reactive::set(b, 2);
/// });
/// assert_eq!(runs.get(), 2);
/// ```
pub fn batch(f: impl FnOnce()) {
    RUNTIME.with(|rt| rt.borrow_mut().batch_depth += 1);
    f();
    RUNTIME.with(|rt| rt.borrow_mut().batch_depth -= 1);
    maybe_flush();
}

// ---------------------------------------------------------------------------
// Owner scopes
// ---------------------------------------------------------------------------

/// An owner scope. Signals and effects created while this scope is active
/// are disposed when the scope is dropped.
///
/// # Example
/// ```
/// # sihora::core::reactive::reset();
/// use std::cell::Cell;
/// use std::rc::Rc;
///
/// let ran = Rc::new(Cell::new(0));
/// let r = ran.clone();
/// {
///     let _scope = sihora::core::reactive::create_scope();
///     let sig = sihora::core::reactive::create_signal(0);
///     sihora::core::reactive::create_effect(move || {
///         let _ = sihora::core::reactive::get(sig);
///         r.set(r.get() + 1);
///     });
///     assert_eq!(ran.get(), 1);
///     sihora::core::reactive::set(sig, 1);
///     assert_eq!(ran.get(), 2);
/// }
/// // scope dropped — effect and signal disposed
/// assert_eq!(ran.get(), 2);
/// ```
pub struct Scope {
    id: ScopeId,
}

impl Drop for Scope {
    fn drop(&mut self) {
        dispose_scope(self.id);
    }
}

/// Create a new owner scope. Signals and effects created after this call
/// (until the returned [`Scope`] is dropped) belong to this scope and
/// are cleaned up when it drops.
pub fn create_scope() -> Scope {
    RUNTIME.with(|rt| {
        let mut rt = rt.borrow_mut();
        let parent = rt.current_scope;
        let id = rt.scopes.insert(ScopeData {
            signals: Vec::new(),
            effects: Vec::new(),
            children: Vec::new(),
            parent,
        });
        if let Some(pid) = parent {
            if let Some(p) = rt.scopes.get_mut(pid) {
                p.children.push(id);
            }
        }
        rt.current_scope = Some(id);
        Scope { id }
    })
}

// ---------------------------------------------------------------------------
// Platform integration
// ---------------------------------------------------------------------------

/// Check and clear the dirty flag. Called by the platform to decide
/// whether a redraw is needed.
pub fn take_dirty() -> bool {
    RUNTIME.with(|rt| {
        let mut rt = rt.borrow_mut();
        let d = rt.dirty;
        rt.dirty = false;
        d
    })
}

/// Reset the runtime to a clean state. For tests only.
#[doc(hidden)]
pub fn reset() {
    RUNTIME.with(|rt| {
        *rt.borrow_mut() = Runtime::new();
    });
}

// ---------------------------------------------------------------------------
// Internal machinery
// ---------------------------------------------------------------------------

fn maybe_flush() {
    let should = RUNTIME.with(|rt| {
        let rt = rt.borrow();
        !rt.flushing && rt.batch_depth == 0 && !rt.pending.is_empty()
    });
    if should {
        flush_effects();
    }
}

fn flush_effects() {
    RUNTIME.with(|rt| rt.borrow_mut().flushing = true);

    for _ in 0..1000 {
        let id = RUNTIME.with(|rt| rt.borrow_mut().pending.pop());
        let Some(id) = id else { break };

        let exists = RUNTIME.with(|rt| rt.borrow().effects.contains_key(id));
        if exists {
            run_effect(id);
        }
    }

    RUNTIME.with(|rt| rt.borrow_mut().flushing = false);
}

fn run_effect(id: EffectId) {
    let func = RUNTIME.with(|rt| {
        let mut rt = rt.borrow_mut();
        if let Some(eff) = rt.effects.get_mut(id) {
            let old_deps = std::mem::take(&mut eff.deps);
            for sig_id in old_deps {
                if let Some(sig) = rt.signals.get_mut(sig_id) {
                    sig.subscribers.retain(|&e| e != id);
                }
            }
        }
        rt.current_effect = Some(id);
        rt.effects[id].func.clone()
    });

    #[cfg(debug_assertions)]
    eprintln!("[sihora::reactive] effect {:?} running", id);

    func();

    RUNTIME.with(|rt| rt.borrow_mut().current_effect = None);
}

fn dispose_scope(id: ScopeId) {
    let to_dispose = RUNTIME.with(|rt| {
        let rt = rt.borrow();
        let mut out = Vec::new();
        collect_scope_tree(&rt, id, &mut out);
        out
    });

    RUNTIME.with(|rt| {
        let mut rt = rt.borrow_mut();
        for sid in to_dispose {
            dispose_scope_inner(&mut rt, sid);
        }
    });
}

fn collect_scope_tree(rt: &Runtime, id: ScopeId, out: &mut Vec<ScopeId>) {
    if let Some(scope) = rt.scopes.get(id) {
        for &child in &scope.children {
            collect_scope_tree(rt, child, out);
        }
    }
    out.push(id);
}

fn dispose_scope_inner(rt: &mut Runtime, id: ScopeId) {
    let Some(scope) = rt.scopes.remove(id) else {
        return;
    };

    for eid in &scope.effects {
        if let Some(eff) = rt.effects.remove(*eid) {
            for sig_id in &eff.deps {
                if let Some(sig) = rt.signals.get_mut(*sig_id) {
                    sig.subscribers.retain(|e| e != eid);
                }
            }
        }
        rt.pending.retain(|e| e != eid);
    }

    for sid in &scope.signals {
        rt.signals.remove(*sid);
    }

    if let Some(pid) = scope.parent {
        if let Some(p) = rt.scopes.get_mut(pid) {
            p.children.retain(|c| *c != id);
        }
    }

    if rt.current_scope == Some(id) {
        rt.current_scope = scope.parent;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn signal_read_write() {
        reset();
        let s = create_signal(42);
        assert_eq!(get(s), 42);
        set(s, 99);
        assert_eq!(get(s), 99);
    }

    #[test]
    fn update_signal() {
        reset();
        let s = create_signal(10);
        update(s, |v| v + 5);
        assert_eq!(get(s), 15);
    }

    #[test]
    fn effect_runs_on_change() {
        reset();
        let s = create_signal(0);
        let log = Rc::new(Cell::new(0));
        let l = log.clone();
        create_effect(move || {
            l.set(get(s));
        });
        assert_eq!(log.get(), 0);
        set(s, 7);
        assert_eq!(log.get(), 7);
        set(s, 100);
        assert_eq!(log.get(), 100);
    }

    #[test]
    fn effect_ignores_unrelated_signal() {
        reset();
        let a = create_signal(1);
        let b = create_signal(2);
        let runs = Rc::new(Cell::new(0));
        let r = runs.clone();
        create_effect(move || {
            let _ = get(a);
            r.set(r.get() + 1);
        });
        assert_eq!(runs.get(), 1);
        set(b, 99);
        assert_eq!(runs.get(), 1);
    }

    #[test]
    fn memo_derived_value() {
        reset();
        let count = create_signal(3);
        let doubled = create_memo(move || get(count) * 2);
        assert_eq!(get(doubled), 6);
        set(count, 10);
        assert_eq!(get(doubled), 20);
    }

    #[test]
    fn memo_skips_equal() {
        reset();
        let input = create_signal(4);
        let clamped = create_memo(move || get(input).min(10));
        let runs = Rc::new(Cell::new(0));
        let r = runs.clone();
        create_effect(move || {
            let _ = get(clamped);
            r.set(r.get() + 1);
        });
        assert_eq!(runs.get(), 1);
        set(input, 5);
        assert_eq!(get(clamped), 5);
        set(input, 100);
        assert_eq!(get(clamped), 10);
        set(input, 200);
        assert_eq!(get(clamped), 10);
    }

    #[test]
    fn batch_defers_effects() {
        reset();
        let a = create_signal(0);
        let b = create_signal(0);
        let runs = Rc::new(Cell::new(0));
        let r = runs.clone();
        create_effect(move || {
            let _ = get(a) + get(b);
            r.set(r.get() + 1);
        });
        assert_eq!(runs.get(), 1);
        batch(|| {
            set(a, 1);
            set(b, 2);
        });
        assert_eq!(runs.get(), 2);
    }

    #[test]
    fn scope_cleanup() {
        reset();
        let ran = Rc::new(Cell::new(0));
        let r = ran.clone();

        {
            let _scope = create_scope();
            let s = create_signal(0);
            create_effect(move || {
                let _ = get(s);
                r.set(r.get() + 1);
            });
            assert_eq!(ran.get(), 1);
            set(s, 1);
            assert_eq!(ran.get(), 2);
        }
        assert_eq!(ran.get(), 2);
    }

    #[test]
    fn nested_scopes() {
        reset();
        let outer_ran = Rc::new(Cell::new(0));
        let inner_ran = Rc::new(Cell::new(0));

        {
            let _outer = create_scope();
            let os = create_signal(0);
            let or = outer_ran.clone();
            create_effect(move || {
                let _ = get(os);
                or.set(or.get() + 1);
            });

            {
                let _inner = create_scope();
                let is = create_signal(0);
                let ir = inner_ran.clone();
                create_effect(move || {
                    let _ = get(is);
                    ir.set(ir.get() + 1);
                });
                assert_eq!(inner_ran.get(), 1);
                set(is, 1);
                assert_eq!(inner_ran.get(), 2);
            }
            assert_eq!(inner_ran.get(), 2);

            assert_eq!(outer_ran.get(), 1);
            set(os, 1);
            assert_eq!(outer_ran.get(), 2);
        }
        assert_eq!(outer_ran.get(), 2);
    }

    #[test]
    fn dirty_flag() {
        reset();
        assert!(!take_dirty());
        let s = create_signal(0);
        set(s, 1);
        assert!(take_dirty());
        assert!(!take_dirty());
    }
}
