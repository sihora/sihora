use std::sync::Arc;

use crate::core::{Event, Key, Modifiers};
use winit::event::WindowEvent;
use winit::window::Window;

pub(crate) fn translate_key(event: &winit::event::KeyEvent) -> Option<Key> {
    use winit::keyboard::Key as WKey;
    use winit::keyboard::NamedKey;

    match &event.logical_key {
        WKey::Named(NamedKey::Space) => Some(Key::Space),
        WKey::Named(NamedKey::Enter) => Some(Key::Enter),
        WKey::Named(NamedKey::Escape) => Some(Key::Escape),
        WKey::Named(NamedKey::Tab) => Some(Key::Tab),
        WKey::Named(NamedKey::Backspace) => Some(Key::Backspace),
        WKey::Named(NamedKey::Delete) => Some(Key::Delete),
        WKey::Named(NamedKey::Home) => Some(Key::Home),
        WKey::Named(NamedKey::End) => Some(Key::End),
        WKey::Named(NamedKey::ArrowUp) => Some(Key::ArrowUp),
        WKey::Named(NamedKey::ArrowDown) => Some(Key::ArrowDown),
        WKey::Named(NamedKey::ArrowLeft) => Some(Key::ArrowLeft),
        WKey::Named(NamedKey::ArrowRight) => Some(Key::ArrowRight),
        WKey::Character(s) => s.chars().next().map(Key::Char),
        _ => None,
    }
}

pub(crate) fn translate_modifiers(state: winit::keyboard::ModifiersState) -> Modifiers {
    Modifiers {
        shift: state.shift_key(),
        ctrl: state.control_key(),
        alt: state.alt_key(),
        meta: state.super_key(),
    }
}

pub(crate) fn process_common_events(
    event: &WindowEvent,
    pending: &mut Vec<Event>,
    cursor_pos: &mut [f32; 2],
    modifiers: &mut Modifiers,
    window: &Option<Arc<Window>>,
) {
    match event {
        WindowEvent::CursorMoved { position, .. } => {
            if let Some(w) = window {
                let s = w.scale_factor() as f32;
                cursor_pos[0] = position.x as f32 / s;
                cursor_pos[1] = position.y as f32 / s;
                pending.push(Event::PointerMoved {
                    x: cursor_pos[0],
                    y: cursor_pos[1],
                });
                w.request_redraw();
            }
        }

        WindowEvent::MouseInput {
            state,
            button: winit::event::MouseButton::Left,
            ..
        } => {
            let ev = match state {
                winit::event::ElementState::Pressed => Event::PointerPressed {
                    x: cursor_pos[0],
                    y: cursor_pos[1],
                },
                winit::event::ElementState::Released => Event::PointerReleased {
                    x: cursor_pos[0],
                    y: cursor_pos[1],
                },
            };
            pending.push(ev);
            if let Some(w) = window {
                w.request_redraw();
            }
        }

        WindowEvent::MouseWheel { delta, .. } => {
            let (dx, dy) = match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => (*x * 20.0, *y * 20.0),
                winit::event::MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
            };
            pending.push(Event::Scroll {
                x: cursor_pos[0],
                y: cursor_pos[1],
                dx,
                dy,
            });
            if let Some(w) = window {
                w.request_redraw();
            }
        }

        WindowEvent::ModifiersChanged(mods) => {
            *modifiers = translate_modifiers(mods.state());
        }

        WindowEvent::KeyboardInput {
            event: ref key_event,
            ..
        } => {
            if let Some(key) = translate_key(key_event) {
                let ev = match key_event.state {
                    winit::event::ElementState::Pressed => Event::KeyPressed {
                        key,
                        modifiers: *modifiers,
                    },
                    winit::event::ElementState::Released => Event::KeyReleased {
                        key,
                        modifiers: *modifiers,
                    },
                };
                pending.push(ev);
                if let Some(w) = window {
                    w.request_redraw();
                }
            }
        }

        WindowEvent::Touch(touch) => {
            if let Some(w) = window {
                let s = w.scale_factor() as f32;
                let x = touch.location.x as f32 / s;
                let y = touch.location.y as f32 / s;
                match touch.phase {
                    winit::event::TouchPhase::Started => {
                        cursor_pos[0] = x;
                        cursor_pos[1] = y;
                        pending.push(Event::PointerPressed { x, y });
                    }
                    winit::event::TouchPhase::Moved => {
                        cursor_pos[0] = x;
                        cursor_pos[1] = y;
                        pending.push(Event::PointerMoved { x, y });
                    }
                    winit::event::TouchPhase::Ended | winit::event::TouchPhase::Cancelled => {
                        pending.push(Event::PointerReleased { x, y });
                    }
                }
                w.request_redraw();
            }
        }

        _ => {}
    }
}
