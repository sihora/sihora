use sihora::a11y::A11yTree;
use sihora::reactive::{create_signal, get, set};
use sihora::text_input::TextEditState;
use sihora::{hit_test, run_app, AppFrame, Color, DrawCommand, Event, Key, TextDraw};

use sihora::data::{Handle, Model};
use sihora::theme::Theme;
use sihora::widgets::{button, checkbox, divider, text, text_field};

#[derive(Clone, PartialEq, Model)]
struct Todo {
    id: i64,
    title: String,
    done: bool,
}

fn main() {
    sihora::data::open("todo.db").expect("failed to open database");
    sihora::data::register::<Todo>().expect("failed to register Todo table");

    let input = TextEditState::new();
    let scroll_y = create_signal(0.0_f32);
    let todos_sig = sihora::data::query_all::<Todo>();

    run_app(move |w, h, text_ctx, events| {
        let theme = Theme::light();
        let sp = &theme.spacing;

        for ev in events {
            match ev {
                Event::Scroll { dy, .. } => {
                    let sy = get(scroll_y);
                    set(scroll_y, (sy - dy * 30.0).max(0.0));
                }
                Event::PointerPressed { x, y } => {
                    let sy = get(scroll_y);

                    // Add button
                    if hit_test(*x, *y, w - 40.0 - 80.0, 20.0 - sy, 80.0, 36.0) {
                        let text = input.text();
                        if !text.is_empty() {
                            sihora::data::insert(Todo {
                                id: 0,
                                title: text,
                                done: false,
                            })
                            .ok();
                            input.clear();
                        }
                    }

                    // Toggle done / delete per row
                    let todos = get(todos_sig);
                    for (i, todo) in todos.iter().enumerate() {
                        let row_y = 80.0 + i as f32 * 48.0 - sy;
                        // Checkbox area
                        if hit_test(*x, *y, 40.0, row_y + 12.0, 20.0, 20.0) {
                            let handle: Handle<Todo> = Handle::from_id(todo.id);
                            let mut updated = todo.clone();
                            updated.done = !updated.done;
                            sihora::data::save(handle, &updated).ok();
                        }
                        // Delete button
                        if hit_test(*x, *y, w - 40.0 - 60.0, row_y + 4.0, 60.0, 36.0) {
                            let handle: Handle<Todo> = Handle::from_id(todo.id);
                            sihora::data::delete(handle).ok();
                        }
                    }
                }
                Event::KeyPressed {
                    key: Key::Enter, ..
                } => {
                    let text = input.text();
                    if !text.is_empty() {
                        sihora::data::insert(Todo {
                            id: 0,
                            title: text,
                            done: false,
                        })
                        .ok();
                        input.clear();
                    }
                }
                Event::KeyPressed { key, modifiers } => match key {
                    Key::Backspace => input.delete_backward(),
                    Key::Delete => input.delete_forward(),
                    Key::ArrowLeft => input.move_left(modifiers.shift),
                    Key::ArrowRight => input.move_right(modifiers.shift),
                    Key::Home => input.move_home(modifiers.shift),
                    Key::End => input.move_end(modifiers.shift),
                    Key::Char(c) => {
                        if modifiers.meta && *c == 'a' {
                            input.select_all();
                        } else if !modifiers.ctrl && !modifiers.meta {
                            input.insert(*c);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let sy = get(scroll_y);
        let todos = get(todos_sig);
        let mut cmds: Vec<DrawCommand> = Vec::with_capacity(64);

        // Background
        cmds.push(DrawCommand::Rect {
            position: [0.0, 0.0],
            size: [w, h],
            color: theme.colors.bg,
            corner_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        });

        let left = 40.0;
        let mut y_cursor = 20.0 - sy;

        // Input row
        cmds.extend(text_field::draw(
            &text_field::Props {
                value: &input.text(),
                placeholder: "Add a todo…",
                state: text_field::State::Normal,
                x: left,
                y: y_cursor,
                width: w - 80.0 - left - sp.sm - 80.0,
            },
            &theme,
            text_ctx,
        ));

        cmds.extend(button::draw(
            &button::Props {
                label: "Add",
                x: w - 40.0 - 80.0,
                y: y_cursor,
                width: 80.0,
                ..Default::default()
            },
            &theme,
            text_ctx,
        ));

        y_cursor += 56.0;
        cmds.extend(divider::draw(
            &divider::Props {
                direction: divider::Direction::Horizontal,
                x: left,
                y: y_cursor,
                length: w - 80.0,
            },
            &theme,
        ));

        y_cursor += sp.md;

        // Todo list
        for todo in &todos {
            let check_val = if todo.done {
                checkbox::Value::Checked
            } else {
                checkbox::Value::Unchecked
            };
            cmds.extend(checkbox::draw(
                &checkbox::Props {
                    value: check_val,
                    state: checkbox::State::Normal,
                    x: left,
                    y: y_cursor + 12.0,
                },
                &theme,
            ));

            let text_color = if todo.done {
                text::TextColor::Disabled
            } else {
                text::TextColor::Primary
            };
            cmds.extend(text::draw(
                &text::Props {
                    text: &todo.title,
                    role: text::Role::Body,
                    color: text_color,
                    x: left + 32.0,
                    y: y_cursor + 14.0,
                },
                &theme,
                text_ctx,
            ));

            cmds.extend(button::draw(
                &button::Props {
                    label: "Delete",
                    variant: button::Variant::Destructive,
                    size: button::Size::Sm,
                    x: w - 40.0 - 60.0,
                    y: y_cursor + 4.0,
                    width: 60.0,
                    ..Default::default()
                },
                &theme,
                text_ctx,
            ));

            y_cursor += 48.0;
        }

        if todos.is_empty() {
            cmds.extend(text::draw(
                &text::Props {
                    text: "No todos yet. Add one above!",
                    role: text::Role::Body,
                    color: text::TextColor::Secondary,
                    x: left,
                    y: y_cursor + 20.0,
                },
                &theme,
                text_ctx,
            ));
        }

        // Footer showing count
        let count_text = format!("{} items", todos.len());
        cmds.push(text_ctx.draw_label(
            &count_text,
            theme.typography.caption,
            left,
            h - 24.0,
            theme.colors.text_secondary,
        ));

        AppFrame {
            commands: cmds,
            a11y: A11yTree::empty("Persistent Todo"),
        }
    });
}
