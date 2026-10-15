//! Animation runtime: springs, transitions, reduce-motion, interruptible.
//!
//! Animated values wrap a reactive [`Signal<f32>`] and smoothly interpolate
//! toward a target each frame. The platform calls [`tick`] once per frame.
//!
//! # Example
//! ```
//! use sihora::core::reactive;
//! use sihora::core::animation::*;
//!
//! reactive::reset();
//! reset();
//!
//! let anim = create_animated(0.0, Spring::default());
//! animate_to(anim, 100.0);
//!
//! // Simulate a few frames
//! for _ in 0..300 {
//!     tick(1.0 / 60.0);
//! }
//! assert!((reactive::get(anim.signal()) - 100.0).abs() < 0.1);
//! ```

use crate::core::reactive::{self, Signal};
use slotmap::SlotMap;
use std::cell::RefCell;

slotmap::new_key_type! {
    struct AnimId;
}

/// Spring physics configuration.
///
/// # Example
/// ```
/// use sihora::core::animation::Spring;
/// let s = Spring::default(); // critically-damped, ~0.3s settle
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Spring {
    /// Restoring force coefficient. Higher = snappier.
    pub stiffness: f32,
    /// Friction coefficient. Higher = less oscillation.
    pub damping: f32,
    /// Inertia. Higher = more momentum.
    pub mass: f32,
}

impl Default for Spring {
    fn default() -> Self {
        Self {
            stiffness: 170.0,
            damping: 26.0,
            mass: 1.0,
        }
    }
}

impl Spring {
    /// Gentle spring for subtle motion.
    pub fn gentle() -> Self {
        Self {
            stiffness: 120.0,
            damping: 14.0,
            mass: 1.0,
        }
    }

    /// Bouncy spring with visible overshoot.
    pub fn bouncy() -> Self {
        Self {
            stiffness: 300.0,
            damping: 10.0,
            mass: 1.0,
        }
    }

    /// Stiff spring that settles quickly.
    pub fn stiff() -> Self {
        Self {
            stiffness: 300.0,
            damping: 30.0,
            mass: 1.0,
        }
    }
}

/// Easing curve for timed transitions.
///
/// # Example
/// ```
/// use sihora::core::animation::Easing;
/// assert!((Easing::Linear.apply(0.5) - 0.5).abs() < f32::EPSILON);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    /// Constant speed.
    Linear,
    /// Starts slow, accelerates.
    EaseIn,
    /// Starts fast, decelerates.
    EaseOut,
    /// Slow start and end.
    EaseInOut,
}

impl Easing {
    /// Map a linear `t` in `[0, 1]` through this curve.
    pub fn apply(self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
        }
    }
}

/// Timed transition configuration.
///
/// # Example
/// ```
/// use sihora::core::animation::Transition;
/// let t = Transition::linear(0.3);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Transition {
    /// Duration in seconds.
    pub duration: f32,
    /// Easing curve.
    pub easing: Easing,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            duration: 0.3,
            easing: Easing::EaseInOut,
        }
    }
}

impl Transition {
    /// Linear transition over `duration` seconds.
    pub fn linear(duration: f32) -> Self {
        Self {
            duration,
            easing: Easing::Linear,
        }
    }

    /// Ease-in transition.
    pub fn ease_in(duration: f32) -> Self {
        Self {
            duration,
            easing: Easing::EaseIn,
        }
    }

    /// Ease-out transition.
    pub fn ease_out(duration: f32) -> Self {
        Self {
            duration,
            easing: Easing::EaseOut,
        }
    }
}

/// The kind of animation driving an [`AnimatedSignal`].
#[derive(Debug, Clone, Copy)]
pub enum AnimationKind {
    /// Physics-based spring.
    Spring(Spring),
    /// Duration-based transition with easing.
    Transition(Transition),
}

impl From<Spring> for AnimationKind {
    fn from(s: Spring) -> Self {
        AnimationKind::Spring(s)
    }
}

impl From<Transition> for AnimationKind {
    fn from(t: Transition) -> Self {
        AnimationKind::Transition(t)
    }
}

/// A reactive signal that animates toward its target value.
///
/// Read the current interpolated value with [`reactive::get`] on the
/// inner signal. Drive it with [`animate_to`].
#[derive(Debug, Clone, Copy)]
pub struct AnimatedSignal {
    signal: Signal<f32>,
    id: AnimId,
}

impl AnimatedSignal {
    /// The underlying reactive signal holding the current value.
    pub fn signal(&self) -> Signal<f32> {
        self.signal
    }

    /// Read the current interpolated value (tracks reactive dependency).
    pub fn get(&self) -> f32 {
        reactive::get(self.signal)
    }

    /// Check whether the animation has settled at its target.
    pub fn is_complete(&self) -> bool {
        ANIM_RT.with(|rt| {
            rt.borrow()
                .animations
                .get(self.id)
                .is_none_or(|s| !s.active)
        })
    }
}

struct AnimState {
    signal: Signal<f32>,
    current: f32,
    target: f32,
    velocity: f32,
    kind: AnimationKind,
    start_value: f32,
    elapsed: f32,
    active: bool,
}

struct AnimRuntime {
    animations: SlotMap<AnimId, AnimState>,
    reduce_motion: bool,
}

impl AnimRuntime {
    fn new() -> Self {
        Self {
            animations: SlotMap::with_key(),
            reduce_motion: false,
        }
    }
}

thread_local! {
    static ANIM_RT: RefCell<AnimRuntime> = RefCell::new(AnimRuntime::new());
}

const SETTLE_POS: f32 = 0.001;
const SETTLE_VEL: f32 = 0.01;

/// Create an animated signal with an initial value and animation kind.
///
/// # Example
/// ```
/// # sihora::core::reactive::reset(); sihora::core::animation::reset();
/// let pos = sihora::core::animation::create_animated(0.0,
///     sihora::core::animation::Spring::default());
/// ```
pub fn create_animated(initial: f32, kind: impl Into<AnimationKind>) -> AnimatedSignal {
    let signal = reactive::create_signal(initial);
    let id = ANIM_RT.with(|rt| {
        rt.borrow_mut().animations.insert(AnimState {
            signal,
            current: initial,
            target: initial,
            velocity: 0.0,
            kind: kind.into(),
            start_value: initial,
            elapsed: 0.0,
            active: false,
        })
    });
    AnimatedSignal { signal, id }
}

/// Set the animation target. Interpolates from the current value.
///
/// Springs retain velocity for smooth retargeting. Transitions restart
/// from the current position. With reduce-motion, snaps immediately.
pub fn animate_to(anim: AnimatedSignal, target: f32) {
    let snap = ANIM_RT.with(|rt| {
        let mut rt = rt.borrow_mut();
        let rm = rt.reduce_motion;
        if let Some(state) = rt.animations.get_mut(anim.id) {
            if rm {
                state.current = target;
                state.target = target;
                state.velocity = 0.0;
                state.active = false;
            } else {
                state.target = target;
                state.active = true;
                if matches!(state.kind, AnimationKind::Transition(_)) {
                    state.start_value = state.current;
                    state.elapsed = 0.0;
                }
            }
        }
        rm
    });

    if snap {
        reactive::set(anim.signal, target);
    }
}

/// Advance all active animations by `dt` seconds.
///
/// Returns `true` if any animation is still running (caller should
/// schedule another frame).
pub fn tick(dt: f32) -> bool {
    let (updates, any_active) = ANIM_RT.with(|rt| {
        let mut rt = rt.borrow_mut();
        let mut any_active = false;
        let mut updates = Vec::new();
        let ids: Vec<AnimId> = rt.animations.keys().collect();

        for id in ids {
            let state = &mut rt.animations[id];
            if !state.active {
                continue;
            }

            match state.kind {
                AnimationKind::Spring(spring) => {
                    let displacement = state.current - state.target;
                    let force = -spring.stiffness * displacement - spring.damping * state.velocity;
                    let accel = force / spring.mass;
                    state.velocity += accel * dt;
                    state.current += state.velocity * dt;

                    if displacement.abs() < SETTLE_POS && state.velocity.abs() < SETTLE_VEL {
                        state.current = state.target;
                        state.velocity = 0.0;
                        state.active = false;
                    }
                }
                AnimationKind::Transition(transition) => {
                    state.elapsed += dt;
                    let t = (state.elapsed / transition.duration).min(1.0);
                    let eased = transition.easing.apply(t);
                    state.current = state.start_value + (state.target - state.start_value) * eased;

                    if t >= 1.0 {
                        state.current = state.target;
                        state.active = false;
                    }
                }
            }

            updates.push((state.signal, state.current));
            if state.active {
                any_active = true;
            }
        }

        (updates, any_active)
    });

    for (signal, value) in updates {
        reactive::set(signal, value);
    }

    any_active
}

/// Enable or disable reduce-motion mode.
///
/// When enabled, [`animate_to`] snaps to the target with no interpolation.
pub fn set_reduce_motion(enabled: bool) {
    ANIM_RT.with(|rt| rt.borrow_mut().reduce_motion = enabled);
}

/// Check whether reduce-motion is enabled.
pub fn reduce_motion() -> bool {
    ANIM_RT.with(|rt| rt.borrow().reduce_motion)
}

/// Reset the animation runtime. For tests only.
#[doc(hidden)]
pub fn reset() {
    ANIM_RT.with(|rt| *rt.borrow_mut() = AnimRuntime::new());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() {
        reactive::reset();
        reset();
    }

    #[test]
    fn spring_converges() {
        setup();
        let anim = create_animated(0.0, Spring::default());
        animate_to(anim, 100.0);

        for _ in 0..300 {
            tick(1.0 / 60.0);
        }

        let val = anim.get();
        assert!(
            (val - 100.0).abs() < 0.01,
            "spring should converge to 100.0, got {val}"
        );
    }

    #[test]
    fn transition_completes() {
        setup();
        let anim = create_animated(0.0, Transition::linear(0.5));
        animate_to(anim, 100.0);

        for _ in 0..60 {
            tick(1.0 / 60.0);
        }

        let val = anim.get();
        assert!(
            (val - 100.0).abs() < 0.1,
            "transition should reach 100.0, got {val}"
        );
    }

    #[test]
    fn transition_midpoint() {
        setup();
        let anim = create_animated(0.0, Transition::linear(1.0));
        animate_to(anim, 100.0);

        tick(0.5);

        let val = anim.get();
        assert!(
            (val - 50.0).abs() < 0.1,
            "linear at 50% should be ~50.0, got {val}"
        );
    }

    #[test]
    fn interruptible_spring() {
        setup();
        let anim = create_animated(0.0, Spring::default());
        animate_to(anim, 100.0);

        for _ in 0..10 {
            tick(1.0 / 60.0);
        }

        let mid = anim.get();
        assert!(mid > 0.0 && mid < 100.0, "should be mid-animation: {mid}");

        animate_to(anim, 0.0);

        for _ in 0..300 {
            tick(1.0 / 60.0);
        }

        let val = anim.get();
        assert!(val.abs() < 0.01, "should return to 0.0, got {val}");
    }

    #[test]
    fn reduce_motion_snaps() {
        setup();
        set_reduce_motion(true);

        let anim = create_animated(0.0, Spring::default());
        animate_to(anim, 100.0);

        let val = anim.get();
        assert!(
            (val - 100.0).abs() < f32::EPSILON,
            "reduce-motion should snap, got {val}"
        );
        assert!(anim.is_complete());
    }

    #[test]
    fn tick_returns_false_when_idle() {
        setup();
        let _anim = create_animated(50.0, Spring::default());
        assert!(!tick(1.0 / 60.0));
    }

    #[test]
    fn tick_returns_true_while_animating() {
        setup();
        let anim = create_animated(0.0, Spring::default());
        animate_to(anim, 100.0);
        assert!(tick(1.0 / 60.0));
    }

    #[test]
    fn easing_boundaries() {
        assert!((Easing::Linear.apply(0.0)).abs() < f32::EPSILON);
        assert!((Easing::Linear.apply(1.0) - 1.0).abs() < f32::EPSILON);
        assert!((Easing::Linear.apply(0.5) - 0.5).abs() < f32::EPSILON);

        assert!(Easing::EaseIn.apply(0.5) < 0.5);
        assert!(Easing::EaseOut.apply(0.5) > 0.5);

        assert!((Easing::EaseInOut.apply(0.0)).abs() < f32::EPSILON);
        assert!((Easing::EaseInOut.apply(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn is_complete_tracks_state() {
        setup();
        let anim = create_animated(0.0, Transition::linear(0.1));
        assert!(anim.is_complete());

        animate_to(anim, 1.0);
        assert!(!anim.is_complete());

        for _ in 0..20 {
            tick(1.0 / 60.0);
        }
        assert!(anim.is_complete());
    }

    #[test]
    fn bouncy_spring_overshoots() {
        setup();
        let anim = create_animated(0.0, Spring::bouncy());
        animate_to(anim, 100.0);

        let mut max_val = 0.0_f32;
        for _ in 0..300 {
            tick(1.0 / 60.0);
            max_val = max_val.max(anim.get());
        }

        assert!(
            max_val > 100.0,
            "bouncy spring should overshoot, max was {max_val}"
        );
        let final_val = anim.get();
        assert!(
            (final_val - 100.0).abs() < 0.01,
            "should still settle at 100.0, got {final_val}"
        );
    }
}
