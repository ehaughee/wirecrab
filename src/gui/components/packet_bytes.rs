use crate::gui::fonts::JETBRAINS_MONO_FAMILY;
use gpui::*;
use gpui_component::{ActiveTheme, StyledExt as _, scroll::Scrollable};

const BYTES_PER_ROW: usize = 16;

/// Displays packet bytes in a Wireshark-style hex + ASCII grid.
#[derive(IntoElement, Clone)]
pub struct PacketBytesView {
    bytes: Option<Vec<u8>>,
}

impl PacketBytesView {
    pub fn new(bytes: Option<&[u8]>) -> Self {
        Self {
            bytes: bytes.map(|slice| slice.to_vec()),
        }
    }

    fn printable_ascii(byte: u8) -> char {
        match byte {
            0x20..=0x7e => byte as char,
            _ => '.',
        }
    }

    fn render_lines(&self, bytes: &[u8], cx: &mut App) -> Scrollable<Div> {
        let mut lines = div()
            .flex()
            .flex_col()
            .gap_y_1()
            .px_3()
            .py_2()
            .text_sm()
            .font_family(JETBRAINS_MONO_FAMILY)
            .text_color(cx.theme().colors.foreground);

        for (row, chunk) in bytes.chunks(BYTES_PER_ROW).enumerate() {
            let offset = row * BYTES_PER_ROW;

            let mut hex_part = String::new();
            for idx in 0..BYTES_PER_ROW {
                if idx == BYTES_PER_ROW / 2 {
                    hex_part.push(' ');
                }

                if let Some(byte) = chunk.get(idx) {
                    hex_part.push_str(&format!("{:02X} ", byte));
                } else {
                    hex_part.push_str("   ");
                }
            }

            let ascii_part: String = chunk.iter().map(|b| Self::printable_ascii(*b)).collect();

            lines = lines.child(
                div()
                    .flex()
                    .gap_x_4()
                    .items_start()
                    .child(
                        div()
                            .w(px(64.0))
                            .text_color(cx.theme().colors.muted_foreground)
                            .child(format!("{offset:06X}")),
                    )
                    .child(div().flex_grow().child(hex_part))
                    .child(
                        div()
                            .w(px(140.0))
                            .text_color(cx.theme().colors.muted_foreground)
                            .child(ascii_part),
                    ),
            );
        }

        lines.scrollable(Axis::Vertical)
    }
}

impl RenderOnce for PacketBytesView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let base = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(cx.theme().colors.background)
            .border_l_1()
            .border_color(cx.theme().colors.border);

        match self.bytes.as_deref() {
            Some(bytes) if !bytes.is_empty() => base
                .child(
                    div()
                        .flex()
                        .items_center()
                        .px_4()
                        .py_2()
                        .border_b_1()
                        .border_color(cx.theme().colors.border)
                        .text_sm()
                        .text_color(cx.theme().colors.muted_foreground)
                        .child(format!("{} bytes", bytes.len()))
                        .child(
                            div()
                                .flex()
                                .gap_x_4()
                                .items_start()
                                .px_3()
                                .text_xs()
                                .text_color(cx.theme().colors.muted_foreground)
                                .child(div().w(px(64.0)).child("Offset"))
                                .child(div().flex_grow().child("Hexadecimal"))
                                .child(div().w(px(140.0)).child("ASCII")),
                        ),
                )
                .child(self.render_lines(bytes, cx)),
            _ => base.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .size_full()
                    .gap_1()
                    .text_color(cx.theme().colors.muted_foreground)
                    .child(
                        div()
                            .text_sm()
                            .child("Select a packet to preview its bytes"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().colors.muted_foreground)
                            .child("Use the packet table on the left to choose a packet."),
                    ),
            ),
        }
    }
}
