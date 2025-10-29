use crate::flow::*;
use iced::{
    Element, Length, Theme,
    widget::{button, column, container, row, scrollable, text},
};

pub fn packet_table<'a, Message>(
    selected_flow: Option<&'a FlowKey>,
    flows: &'a std::collections::HashMap<FlowKey, Flow>,
    on_close: fn() -> Message,
) -> Element<'a, Message>
where
    Message: Clone + 'static,
{
    if let Some(selected_key) = selected_flow {
        if let Some(flow) = flows.get(selected_key) {
            // Create close button
            let close_button = button(text("✕ Close"))
                .on_press(on_close())
                .style(|theme: &Theme, status| {
                    if matches!(status, button::Status::Hovered) {
                        button::Style {
                            background: Some(theme.extended_palette().danger.base.color.into()),
                            text_color: theme.extended_palette().danger.base.text,
                            border: iced::Border::default(),
                            shadow: iced::Shadow::default(),
                        }
                    } else {
                        button::Style {
                            background: Some(
                                theme.extended_palette().secondary.base.color.into(),
                            ),
                            text_color: theme.extended_palette().secondary.base.text,
                            border: iced::Border::default(),
                            shadow: iced::Shadow::default(),
                        }
                    }
                });

            // Create packet table header
            let header = row![
                text("Packet #").width(Length::FillPortion(1)),
                text("Timestamp").width(Length::FillPortion(2)),
                text("Size (bytes)").width(Length::FillPortion(1)),
                text("Data Preview").width(Length::FillPortion(3)),
            ]
            .spacing(10)
            .padding(10);

            let mut packet_rows = column![].spacing(2);

            // Add close button and title
            let title_row = row![
                text(format!("Packets for Flow: {}", selected_key.to_display())).size(16),
                close_button
            ]
            .spacing(10)
            .padding(10);

            packet_rows = packet_rows.push(title_row);

            // Add header
            let styled_header = container(header).padding(5);
            packet_rows = packet_rows.push(styled_header);

            // Add packet data rows
            for (index, packet) in flow.packets.iter().enumerate() {
                let packet_size = packet.len();

                // Create a hex preview of first 16 bytes
                let preview = if packet.len() > 16 {
                    format!(
                        "{} ...",
                        packet[..16]
                            .iter()
                            .map(|b| format!("{:02x}", b))
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                } else {
                    packet
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" ")
                };

                let packet_row = row![
                    text((index + 1).to_string()).width(Length::FillPortion(1)),
                    text("N/A").width(Length::FillPortion(2)), // No individual packet timestamp
                    text(packet_size.to_string()).width(Length::FillPortion(1)),
                    text(preview).width(Length::FillPortion(3)),
                ]
                .spacing(10)
                .padding(5);

                let styled_packet_row = container(packet_row).padding(2);
                packet_rows = packet_rows.push(styled_packet_row);
            }

            scrollable(packet_rows)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            container(text("Selected flow not found"))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        }
    } else {
        container(text("Select a flow to view packet details"))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}