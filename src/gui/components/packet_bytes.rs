use crate::gui::fonts::JETBRAINS_MONO_FAMILY;
use gpui::*;
use gpui_component::ActiveTheme;

const BYTES_PER_ROW: usize = 16;

// Fixed column widths for consistent alignment
const OFFSET_WIDTH: f32 = 72.0;
const HEX_WIDTH: f32 = 450.0;
const ASCII_WIDTH: f32 = 140.0;

/// Displays packet bytes in a Wireshark-style hex + ASCII grid.
#[derive(IntoElement)]
pub struct PacketBytesView {
    list_state: Option<ListState>,
    bytes: Option<Vec<u8>>,
}

impl PacketBytesView {
    pub fn new(list_state: Option<ListState>, bytes: Option<Vec<u8>>) -> Self {
        Self { list_state, bytes }
    }

    pub fn create_list_state(bytes: &[u8]) -> ListState {
        let row_count = (bytes.len() + BYTES_PER_ROW - 1) / BYTES_PER_ROW;
        tracing::info!("Creating list state with {} rows for {} bytes", row_count, bytes.len());
        ListState::new(
            row_count,
            ListAlignment::Top,
            px(20.0),
        )
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
            .w_full() // Ensure row takes full width
            .px_3()
            .py_px()
            .gap_2()
            .child(div().w(px(OFFSET_WIDTH)).child(format!("{offset:06X}")))
            .child(div().w(px(HEX_WIDTH)).child(hex_part))
            .child(div().w(px(ASCII_WIDTH)).child(ascii_part))
    }
}

impl RenderOnce for PacketBytesView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        tracing::info!("Rendering PacketBytesView. Has state: {}, Has bytes: {}", self.list_state.is_some(), self.bytes.is_some());
        let base = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(cx.theme().colors.background)
            .border_1()
            .border_color(cx.theme().colors.border);

        match (self.list_state, self.bytes) {
            (Some(list_state), Some(bytes)) => {
                base
                    .child(Self::render_header(cx))
                    .child(
                    div()
                        .font_family(JETBRAINS_MONO_FAMILY)
                        .text_sm()
                        .text_color(cx.theme().colors.foreground)
                        .flex_1()
                        .size_full()
                        .child(
                            list(list_state, move |ix, _window, _cx| {
                                let start = ix * BYTES_PER_ROW;
                                let end = (start + BYTES_PER_ROW).min(bytes.len());
                                let chunk = &bytes[start..end];
                                tracing::info!("Rendering row {}", ix);
                                Self::render_row(start, chunk)
                                    .h(px(20.0))
                                    .into_any_element()
                            })
                            .size_full() // Ensure list takes full size of container
                        ),
                )
            }
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
