use sihora::a11y::A11yTree;
use sihora::reactive::{create_signal, get, set};
use sihora::{hit_test, run_app, AppFrame, Color, DrawCommand, Event, Key, TextDraw};

use sihora_theme::{ColorScheme, Theme};
use sihora_widgets::{
    button, card, checkbox, dialog, divider, icon, list_row, text, text_field, toggle,
};

fn main() {
    let scroll_y = create_signal(0.0_f32);
    let dark_mode = create_signal(false);
    let show_dialog = create_signal(false);

    run_app(move |w, h, text_ctx, events| {
        for ev in events {
            match ev {
                Event::Scroll { dy, .. } => {
                    let sy = get(scroll_y);
                    set(scroll_y, (sy - dy * 30.0).max(0.0));
                }
                Event::PointerPressed { x, y } => {
                    let sy = get(scroll_y);
                    if hit_test(*x, *y, w - 80.0, 16.0 - sy, 60.0, 28.0) {
                        set(dark_mode, !get(dark_mode));
                    }
                    if hit_test(*x, *y, 40.0, 880.0 - sy, 120.0, 36.0) {
                        set(show_dialog, true);
                    }
                    if get(show_dialog) {
                        set(show_dialog, false);
                    }
                }
                Event::KeyPressed {
                    key: Key::Escape, ..
                } => {
                    set(show_dialog, false);
                }
                _ => {}
            }
        }

        let scheme = if get(dark_mode) {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        };
        let theme = Theme::new(scheme);
        let sy = get(scroll_y);

        let mut cmds: Vec<DrawCommand> = Vec::with_capacity(128);

        cmds.push(DrawCommand::Rect {
            position: [0.0, 0.0],
            size: [w, h],
            color: theme.colors.bg,
            corner_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        });

        let mut y_cursor = 24.0 - sy;
        let left = 40.0;

        // --- Header ---
        cmds.push(text_ctx.draw_label(
            "Sihora Gallery",
            theme.typography.headline,
            left,
            y_cursor,
            theme.colors.text_primary,
        ));

        let mode_label = if get(dark_mode) { "Light" } else { "Dark" };
        cmds.extend(button::draw(
            &button::Props {
                label: mode_label,
                variant: button::Variant::Secondary,
                size: button::Size::Sm,
                x: w - 80.0,
                y: y_cursor,
                width: 60.0,
                ..Default::default()
            },
            &theme,
            text_ctx,
        ));

        y_cursor += 48.0;
        cmds.extend(divider::draw(
            &divider::Props {
                direction: divider::Direction::Horizontal,
                x: left,
                y: y_cursor,
                length: w - 80.0,
            },
            &theme,
        ));

        // --- Text ---
        y_cursor += 24.0;
        section_header(&mut cmds, text_ctx, &theme, left, y_cursor, "Text");
        y_cursor += 32.0;

        let roles = [
            ("Caption", text::Role::Caption),
            ("Body Small", text::Role::BodySm),
            ("Body", text::Role::Body),
            ("Label", text::Role::Label),
            ("Title Small", text::Role::TitleSm),
            ("Title", text::Role::Title),
            ("Headline", text::Role::Headline),
        ];
        for (label, role) in &roles {
            cmds.extend(text::draw(
                &text::Props {
                    text: label,
                    role: *role,
                    color: text::TextColor::Primary,
                    x: left,
                    y: y_cursor,
                },
                &theme,
                text_ctx,
            ));
            y_cursor += 28.0;
        }

        cmds.extend(text::draw(
            &text::Props {
                text: "Secondary text",
                role: text::Role::Body,
                color: text::TextColor::Secondary,
                x: left,
                y: y_cursor,
            },
            &theme,
            text_ctx,
        ));
        y_cursor += 24.0;

        cmds.extend(text::draw(
            &text::Props {
                text: "Disabled text",
                role: text::Role::Body,
                color: text::TextColor::Disabled,
                x: left,
                y: y_cursor,
            },
            &theme,
            text_ctx,
        ));
        y_cursor += 24.0;

        cmds.extend(text::draw(
            &text::Props {
                text: "Danger text",
                role: text::Role::Body,
                color: text::TextColor::Danger,
                x: left,
                y: y_cursor,
            },
            &theme,
            text_ctx,
        ));
        y_cursor += 24.0;

        cmds.extend(text::draw(
            &text::Props {
                text: "Success text",
                role: text::Role::Body,
                color: text::TextColor::Success,
                x: left,
                y: y_cursor,
            },
            &theme,
            text_ctx,
        ));

        // --- Icon ---
        y_cursor += 40.0;
        section_header(&mut cmds, text_ctx, &theme, left, y_cursor, "Icon");
        y_cursor += 32.0;

        let mut ix = left;
        for size in [icon::Size::Sm, icon::Size::Md, icon::Size::Lg] {
            cmds.extend(icon::draw(
                &icon::Props {
                    size,
                    x: ix,
                    y: y_cursor,
                },
                &theme,
            ));
            ix += 32.0;
        }

        // --- Button ---
        y_cursor += 40.0;
        section_header(&mut cmds, text_ctx, &theme, left, y_cursor, "Button");
        y_cursor += 32.0;

        let btn_w = 130.0;
        let variants = [
            ("Primary", button::Variant::Primary),
            ("Secondary", button::Variant::Secondary),
            ("Ghost", button::Variant::Ghost),
            ("Destructive", button::Variant::Destructive),
        ];
        for (label, variant) in &variants {
            cmds.extend(button::draw(
                &button::Props {
                    label,
                    variant: *variant,
                    x: left,
                    y: y_cursor,
                    width: btn_w,
                    ..Default::default()
                },
                &theme,
                text_ctx,
            ));
            y_cursor += 44.0;
        }

        cmds.extend(button::draw(
            &button::Props {
                label: "Focused",
                state: button::State::Focused,
                x: left,
                y: y_cursor,
                width: btn_w,
                ..Default::default()
            },
            &theme,
            text_ctx,
        ));
        y_cursor += 44.0;

        cmds.extend(button::draw(
            &button::Props {
                label: "Disabled",
                state: button::State::Disabled,
                x: left,
                y: y_cursor,
                width: btn_w,
                ..Default::default()
            },
            &theme,
            text_ctx,
        ));

        // --- TextField ---
        y_cursor += 56.0;
        section_header(&mut cmds, text_ctx, &theme, left, y_cursor, "TextField");
        y_cursor += 32.0;

        let field_w = 240.0;
        let fields = [
            ("Hello world", "", text_field::State::Normal),
            ("", "Placeholder…", text_field::State::Normal),
            ("Focused", "", text_field::State::Focused),
            ("Error state", "", text_field::State::Error),
            ("Disabled", "", text_field::State::Disabled),
        ];
        for (val, ph, state) in &fields {
            cmds.extend(text_field::draw(
                &text_field::Props {
                    value: val,
                    placeholder: ph,
                    state: *state,
                    x: left,
                    y: y_cursor,
                    width: field_w,
                },
                &theme,
                text_ctx,
            ));
            y_cursor += 48.0;
        }

        // --- Toggle ---
        y_cursor += 8.0;
        section_header(&mut cmds, text_ctx, &theme, left, y_cursor, "Toggle");
        y_cursor += 32.0;

        let toggles = [
            (false, toggle::State::Normal),
            (true, toggle::State::Normal),
            (true, toggle::State::Focused),
            (false, toggle::State::Disabled),
        ];
        let mut tx = left;
        for (on, state) in &toggles {
            cmds.extend(toggle::draw(
                &toggle::Props {
                    on: *on,
                    state: *state,
                    x: tx,
                    y: y_cursor,
                },
                &theme,
            ));
            tx += 56.0;
        }

        // --- Checkbox ---
        y_cursor += 40.0;
        section_header(&mut cmds, text_ctx, &theme, left, y_cursor, "Checkbox");
        y_cursor += 32.0;

        let checks = [
            (checkbox::Value::Unchecked, checkbox::State::Normal),
            (checkbox::Value::Checked, checkbox::State::Normal),
            (checkbox::Value::Indeterminate, checkbox::State::Normal),
            (checkbox::Value::Checked, checkbox::State::Focused),
            (checkbox::Value::Unchecked, checkbox::State::Disabled),
        ];
        let mut cx = left;
        for (val, state) in &checks {
            cmds.extend(checkbox::draw(
                &checkbox::Props {
                    value: *val,
                    state: *state,
                    x: cx,
                    y: y_cursor,
                },
                &theme,
            ));
            cx += 32.0;
        }

        // --- Card ---
        y_cursor += 40.0;
        section_header(&mut cmds, text_ctx, &theme, left, y_cursor, "Card");
        y_cursor += 32.0;

        cmds.extend(card::draw(
            &card::Props {
                variant: card::Variant::Flat,
                x: left,
                y: y_cursor,
                width: 200.0,
                height: 80.0,
            },
            &theme,
        ));
        cmds.push(text_ctx.draw_label(
            "Flat card",
            theme.typography.body,
            left + 16.0,
            y_cursor + 30.0,
            theme.colors.text_primary,
        ));

        cmds.extend(card::draw(
            &card::Props {
                variant: card::Variant::Raised,
                x: left + 220.0,
                y: y_cursor,
                width: 200.0,
                height: 80.0,
            },
            &theme,
        ));
        cmds.push(text_ctx.draw_label(
            "Raised card",
            theme.typography.body,
            left + 236.0,
            y_cursor + 30.0,
            theme.colors.text_primary,
        ));

        // --- ListRow ---
        y_cursor += 100.0;
        section_header(&mut cmds, text_ctx, &theme, left, y_cursor, "ListRow");
        y_cursor += 32.0;

        let row_w = w - 80.0;
        let rows = [
            ("Inbox", "12", list_row::State::Normal),
            ("Sent", "3", list_row::State::Hover),
            ("Drafts", "1", list_row::State::Selected),
            ("Trash", "0", list_row::State::Pressed),
        ];
        for (label, trailing, state) in &rows {
            cmds.extend(list_row::draw(
                &list_row::Props {
                    label,
                    trailing,
                    state: *state,
                    x: left,
                    y: y_cursor,
                    width: row_w,
                },
                &theme,
                text_ctx,
            ));
            y_cursor += 46.0;
        }

        // --- Divider ---
        y_cursor += 8.0;
        section_header(&mut cmds, text_ctx, &theme, left, y_cursor, "Divider");
        y_cursor += 32.0;
        cmds.extend(divider::draw(
            &divider::Props {
                direction: divider::Direction::Horizontal,
                x: left,
                y: y_cursor,
                length: 300.0,
            },
            &theme,
        ));
        y_cursor += 16.0;
        cmds.extend(divider::draw(
            &divider::Props {
                direction: divider::Direction::Vertical,
                x: left,
                y: y_cursor,
                length: 40.0,
            },
            &theme,
        ));

        // --- Dialog trigger ---
        y_cursor += 56.0;
        section_header(&mut cmds, text_ctx, &theme, left, y_cursor, "Dialog");
        y_cursor += 32.0;
        cmds.extend(button::draw(
            &button::Props {
                label: "Open Dialog",
                x: left,
                y: y_cursor,
                width: 120.0,
                ..Default::default()
            },
            &theme,
            text_ctx,
        ));

        // --- Dialog overlay ---
        if get(show_dialog) {
            cmds.extend(dialog::draw(
                &dialog::Props {
                    title: "Confirm action",
                    body: "Are you sure you want to proceed?",
                    confirm_label: "OK",
                    cancel_label: "Cancel",
                    viewport_width: w,
                    viewport_height: h,
                },
                &theme,
                text_ctx,
            ));
        }

        AppFrame {
            commands: cmds,
            a11y: A11yTree::empty("Sihora Gallery"),
        }
    });
}

fn section_header(
    cmds: &mut Vec<DrawCommand>,
    text_ctx: &mut sihora::TextContext,
    theme: &Theme,
    x: f32,
    y: f32,
    label: &str,
) {
    cmds.push(text_ctx.draw_label(label, theme.typography.title_sm, x, y, theme.colors.accent));
}
