use crate::flow::*;
use crate::gui::widgets::helpers::{format_ip_address, format_protocol};
use iced::{
    Element, Length, Theme,
    widget::{button, column, container, row, scrollable, text},
};

pub fn flow_table<Message>(
    filtered_flows: &[(FlowKey, Flow)],
    selected_flow: Option<FlowKey>,
    on_flow_selected: fn(FlowKey) -> Message,
) -> Element<Message>
where
    Message: Clone + 'static,
{
    // Create table header
    let header = row![
        text("Timestamp").width(Length::FillPortion(2)),
        text("Source IP").width(Length::FillPortion(2)),
        text("Src Port").width(Length::FillPortion(1)),
        text("Destination IP").width(Length::FillPortion(2)),
        text("Dst Port").width(Length::FillPortion(1)),
        text("Protocol").width(Length::FillPortion(1)),
        text("Packets").width(Length::FillPortion(1)),
        text("Bytes").width(Length::FillPortion(1)),
    ]
    .spacing(10)
    .padding(10);

    // Create table rows
    let mut rows = column![].spacing(2);

    // Add header with styling
    let styled_header = container(header).padding(5);
    rows = rows.push(styled_header);

    // Add data rows
    for (flow_key, flow) in filtered_flows {
        let timestamp_str = format!("{:.6}", flow.timestamp);
        let src_ip_str = format_ip_address(&flow.src_ip);
        let dst_ip_str = format_ip_address(&flow.dst_ip);
        let src_port_str = flow.src_port.map_or("N/A".to_string(), |p| p.to_string());
        let dst_port_str = flow.dst_port.map_or("N/A".to_string(), |p| p.to_string());
        let protocol_str = format_protocol(&flow.protocol);
        let packet_count = flow.packets.len();
        let byte_count: usize = flow.packets.iter().map(|p| p.len()).sum();

        let data_row = row![
            text(timestamp_str).width(Length::FillPortion(2)),
            text(src_ip_str).width(Length::FillPortion(2)),
            text(src_port_str).width(Length::FillPortion(1)),
            text(dst_ip_str).width(Length::FillPortion(2)),
            text(dst_port_str).width(Length::FillPortion(1)),
            text(protocol_str).width(Length::FillPortion(1)),
            text(packet_count.to_string()).width(Length::FillPortion(1)),
            text(byte_count.to_string()).width(Length::FillPortion(1)),
        ]
        .spacing(10)
        .padding(5);

        // Make the row clickable and highlight if selected
        let is_selected = selected_flow == Some(*flow_key);
        let flow_key_clone = *flow_key;
        let styled_row = button(data_row)
            .on_press(on_flow_selected(flow_key_clone))
            .style(move |theme: &Theme, status| {
                if is_selected {
                    button::Style {
                        background: Some(theme.extended_palette().primary.weak.color.into()),
                        text_color: theme.extended_palette().primary.weak.text,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                    }
                } else if matches!(status, button::Status::Hovered) {
                    button::Style {
                        background: Some(theme.extended_palette().background.strong.color.into()),
                        text_color: theme.extended_palette().background.strong.text,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                    }
                } else {
                    button::Style {
                        background: Some(theme.extended_palette().background.base.color.into()),
                        text_color: theme.extended_palette().background.base.text,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                    }
                }
            })
            .width(Length::Fill);

        rows = rows.push(styled_row);
    }

    // Wrap in scrollable container
    scrollable(rows)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
