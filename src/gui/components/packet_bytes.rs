use crate::gui::fonts::JETBRAINS_MONO_FAMILY;
use gpui::*;
use gpui_component::{ActiveTheme, StyledExt};

const BYTES_PER_ROW: usize = 16;

// Fixed column widths for consistent alignment
const OFFSET_WIDTH: f32 = 72.0;
const HEX_WIDTH: f32 = 450.0;
const ASCII_WIDTH: f32 = 140.0;

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

    fn render_header(cx: &mut App) -> Div {
        div()
            .flex()
            .flex_row()
            .flex_shrink_0()
            .px_3()
            .py_1()
            .gap_2()
            .border_b_1()
            .border_color(cx.theme().colors.border)
            .text_xs()
            .text_color(cx.theme().colors.muted_foreground)
            .child(div().w(px(OFFSET_WIDTH)).child("Offset"))
            .child(div().w(px(HEX_WIDTH)).child("Hexadecimal"))
            .child(div().w(px(ASCII_WIDTH)).child("ASCII"))
    }

    fn render_row(offset: usize, chunk: &[u8]) -> Div {
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

        div()
            .flex()
            .flex_row()
            .px_3()
            .py_px()
            .gap_2()
            .child(div().w(px(OFFSET_WIDTH)).child(format!("{offset:06X}")))
            .child(div().w(px(HEX_WIDTH)).child(hex_part))
            .child(div().w(px(ASCII_WIDTH)).child(ascii_part))
    }

    fn render_lines(&self, bytes: &[u8], cx: &mut App) -> Div {
        let rows: Vec<Div> = bytes
            .chunks(BYTES_PER_ROW)
            .enumerate()
            .map(|(row, chunk)| Self::render_row(row * BYTES_PER_ROW, chunk))
            .collect();

        div()
            .flex()
            .flex_col()
            .min_h_0()
            .flex_1()
            .overflow_hidden()
            .child(Self::render_header(cx))
            .child(
                div()
                    .py_1()
                    .font_family(JETBRAINS_MONO_FAMILY)
                    .text_sm()
                    .min_h_0()
                    .flex_1()
                    .overflow_hidden()
                    .children(rows)
                    .scrollable(Axis::Vertical),
            )
    }
}

impl RenderOnce for PacketBytesView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let base = div()
            .flex()
            .flex_col()
            .size_full()
            .min_h_0()
            .overflow_hidden()
            .bg(cx.theme().colors.background)
            .border_l_1()
            .border_color(cx.theme().colors.border);

        match self.bytes.as_deref() {
            Some(bytes) if !bytes.is_empty() => base.child(self.render_lines(bytes, cx)),
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
