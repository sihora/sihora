use sihora::{run_with_text, Color, DrawCommand, TextBuffer};

fn main() {
    let mut buffer: Option<TextBuffer> = None;
    let mut last_width: f32 = 0.0;

    let text = "The quick brown fox jumps over the lazy dog. \
                Sihora renders text with a glyph atlas and instanced quads.\n\n\
                CJK: \u{4F60}\u{597D}\u{4E16}\u{754C} \u{3053}\u{3093}\u{306B}\u{3061}\u{306F} \
                \u{C548}\u{B155}\u{D558}\u{C138}\u{C694}\n\n\
                Emoji: \u{1F680}\u{1F30D}\u{2728}\u{1F525}\u{1F389}";

    run_with_text(move |w, _h, ctx| {
        let max_w = w - 40.0;

        if buffer.is_none() || (last_width - max_w).abs() > 1.0 {
            let mut buf = ctx.create_buffer(text, 20.0, 28.0, Some(max_w));
            if last_width > 0.0 {
                ctx.resize_buffer(&mut buf, Some(max_w));
            }
            last_width = max_w;
            buffer = Some(buf);
        }

        let buf = buffer.as_ref().expect("buffer created above");
        let (tw, th) = buf.measure();

        let bg = DrawCommand::Rect {
            position: [16.0, 16.0],
            size: [tw + 16.0, th + 16.0],
            color: Color::rgba(0.12, 0.12, 0.15, 1.0),
            corner_radius: 8.0,
            border_width: 1.0,
            border_color: Color::rgba(0.3, 0.3, 0.35, 1.0),
        };
        let txt = ctx.draw_text(buf, 24.0, 24.0, Color::WHITE);

        vec![bg, txt]
    });
}
