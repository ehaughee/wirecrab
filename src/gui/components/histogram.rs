use crate::flow::{Flow, FlowKey, Protocol};
use gpui::*;
use gpui_component::plot::scale::{Scale, ScaleBand, ScaleLinear};
use gpui_component::plot::shape::Bar;
use gpui_component::plot::{AxisText, Grid, Plot, PlotAxis, AXIS_GAP};
use gpui_component::{ActiveTheme, Icon, IconName, PixelsExt, StyledExt, h_flex, v_flex};
use gpui_component_macros::IntoPlot;

/// Number of time buckets to divide the capture into.
const DEFAULT_BUCKET_COUNT: usize = 30;

/// A single time bucket aggregating packet counts by protocol.
#[derive(Clone)]
pub struct HistogramBucket {
    pub label: String,
    pub tcp: f64,
    pub udp: f64,
    pub other: f64,
}

/// Protocol categories for the legend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolCategory {
    Tcp,
    Udp,
    Other,
}

#[derive(Clone)]
struct BucketSegment {
    label: SharedString,
    start: f64,
    end: f64,
    color: Hsla,
}

#[derive(IntoPlot)]
struct StackedBarChart {
    data: Vec<HistogramBucket>,
    colors: [Hsla; 3],
    tick_margin: usize,
}

impl StackedBarChart {
    fn new(data: Vec<HistogramBucket>, colors: [Hsla; 3]) -> Self {
        Self {
            data,
            colors,
            tick_margin: 1,
        }
    }

    fn tick_margin(mut self, tick_margin: usize) -> Self {
        self.tick_margin = tick_margin;
        self
    }
}

impl Plot for StackedBarChart {
    fn paint(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        if self.data.is_empty() {
            return;
        }

        let width = bounds.size.width.as_f32();
        let height = bounds.size.height.as_f32() - AXIS_GAP;

        let labels: Vec<SharedString> = self
            .data
            .iter()
            .map(|bucket| SharedString::from(bucket.label.clone()))
            .collect();

        let x_scale = ScaleBand::new(labels.clone(), vec![0., width])
            .padding_inner(0.3)
            .padding_outer(0.15);
        let band_width = x_scale.band_width();

        let totals: Vec<f64> = self
            .data
            .iter()
            .map(|bucket| bucket.tcp + bucket.udp + bucket.other)
            .collect();
        let y_scale = ScaleLinear::new(
            totals
                .iter()
                .copied()
                .chain(std::iter::once(0.0))
                .collect::<Vec<f64>>(),
            vec![height, 10.],
        );

        let x_labels = labels
            .iter()
            .enumerate()
            .filter_map(|(i, label)| {
                if (i + 1) % self.tick_margin == 0 {
                    x_scale.tick(label).map(|x_tick| {
                        AxisText::new(label.clone(), x_tick + band_width / 2., cx.theme().muted_foreground)
                            .align(TextAlign::Center)
                    })
                } else {
                    None
                }
            });

        PlotAxis::new()
            .x(height)
            .x_label(x_labels)
            .stroke(cx.theme().border)
            .paint(&bounds, window, cx);

        Grid::new()
            .y((0..=3).map(|i| height * i as f32 / 4.0).collect())
            .stroke(cx.theme().border)
            .dash_array(&[px(4.), px(2.)])
            .paint(&bounds, window);

        let segments: Vec<BucketSegment> = self
            .data
            .iter()
            .flat_map(|bucket| {
                let mut start = 0.0;
                let mut parts = Vec::with_capacity(3);

                let tcp_end = start + bucket.tcp;
                parts.push(BucketSegment {
                    label: SharedString::from(bucket.label.clone()),
                    start,
                    end: tcp_end,
                    color: self.colors[0],
                });
                start = tcp_end;

                let udp_end = start + bucket.udp;
                parts.push(BucketSegment {
                    label: SharedString::from(bucket.label.clone()),
                    start,
                    end: udp_end,
                    color: self.colors[1],
                });
                start = udp_end;

                let other_end = start + bucket.other;
                parts.push(BucketSegment {
                    label: SharedString::from(bucket.label.clone()),
                    start,
                    end: other_end,
                    color: self.colors[2],
                });

                parts
            })
            .collect();

        let x_for_bar = x_scale.clone();
        let y_for_bar = y_scale.clone();
        let y_for_bar_2 = y_scale.clone();

        Bar::new()
            .data(&segments)
            .band_width(band_width)
            .x(move |seg| x_for_bar.tick(&seg.label))
            .y0(move |seg| y_for_bar.tick(&seg.start).unwrap_or(height))
            .y1(move |seg| y_for_bar_2.tick(&seg.end))
            .fill(|seg| seg.color)
            .paint(&bounds, window, cx);
    }
}

impl ProtocolCategory {
    pub fn label(&self) -> &'static str {
        match self {
            ProtocolCategory::Tcp => "TCP",
            ProtocolCategory::Udp => "UDP",
            ProtocolCategory::Other => "Other",
        }
    }

    pub fn filter_value(&self) -> &'static str {
        match self {
            ProtocolCategory::Tcp => "tcp",
            ProtocolCategory::Udp => "udp",
            ProtocolCategory::Other => "other",
        }
    }

    pub fn all() -> &'static [ProtocolCategory] {
        &[
            ProtocolCategory::Tcp,
            ProtocolCategory::Udp,
            ProtocolCategory::Other,
        ]
    }
}

/// Compute histogram buckets from a set of flows.
pub fn compute_histogram(
    flows: &[(FlowKey, Flow)],
    start_timestamp: Option<f64>,
    bucket_count: usize,
) -> Vec<HistogramBucket> {
    if flows.is_empty() {
        return Vec::new();
    }

    // Collect all packets with their protocol
    let mut all_packets: Vec<(f64, Protocol)> = Vec::new();
    for (_, flow) in flows {
        for packet in &flow.packets {
            all_packets.push((packet.timestamp, flow.protocol));
        }
    }

    if all_packets.is_empty() {
        return Vec::new();
    }

    // Find time range
    let min_ts = start_timestamp.unwrap_or_else(|| {
        all_packets
            .iter()
            .map(|(ts, _)| *ts)
            .fold(f64::INFINITY, f64::min)
    });
    let max_ts = all_packets
        .iter()
        .map(|(ts, _)| *ts)
        .fold(f64::NEG_INFINITY, f64::max);

    let duration = max_ts - min_ts;
    if duration <= 0.0 {
        // All packets at same timestamp
        let mut tcp = 0.0;
        let mut udp = 0.0;
        let mut other = 0.0;
        for (_, proto) in &all_packets {
            match proto {
                Protocol::TCP => tcp += 1.0,
                Protocol::UDP => udp += 1.0,
                Protocol::Other(_) => other += 1.0,
            }
        }
        return vec![HistogramBucket {
            label: "0s".to_string(),
            tcp,
            udp,
            other,
        }];
    }

    let bucket_width = duration / bucket_count as f64;

    // Initialize buckets
    let mut buckets: Vec<HistogramBucket> = (0..bucket_count)
        .map(|i| {
            let bucket_start = i as f64 * bucket_width;
            HistogramBucket {
                label: format!("{:.1}s", bucket_start),
                tcp: 0.0,
                udp: 0.0,
                other: 0.0,
            }
        })
        .collect();

    // Populate buckets
    for (ts, proto) in &all_packets {
        let relative = ts - min_ts;
        let bucket_idx = ((relative / bucket_width).floor() as usize).min(bucket_count - 1);
        match proto {
            Protocol::TCP => buckets[bucket_idx].tcp += 1.0,
            Protocol::UDP => buckets[bucket_idx].udp += 1.0,
            Protocol::Other(_) => buckets[bucket_idx].other += 1.0,
        }
    }

    buckets
}

/// Convenience function to create histogram with default bucket count.
pub fn histogram_from_flows(
    flows: &[(FlowKey, Flow)],
    start_timestamp: Option<f64>,
) -> Vec<HistogramBucket> {
    compute_histogram(flows, start_timestamp, DEFAULT_BUCKET_COUNT)
}

/// Render the protocol histogram chart with header and legend.
pub fn render_histogram(
    buckets: Vec<HistogramBucket>,
    collapsed: bool,
    on_toggle: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_legend_click: impl Fn(ProtocolCategory, &mut Window, &mut App) + 'static,
    cx: &App,
) -> impl IntoElement {
    // Use distinctive semantic colors instead of similar chart blues
    let tcp_color = cx.theme().info; // Blue for TCP
    let udp_color = cx.theme().success; // Green for UDP  
    let other_color = cx.theme().warning; // Orange for Other

    let categories = ProtocolCategory::all();
    let colors = [tcp_color, udp_color, other_color];

    // Build header with collapse toggle and legend
    let collapse_icon = if collapsed {
        IconName::ChevronRight
    } else {
        IconName::ChevronDown
    };

    let header = div()
        .id("histogram_header")
        .flex()
        .items_center()
        .justify_between()
        .w_full()
        .px_3()
        .py_2()
        .bg(cx.theme().colors.secondary)
        .border_b_1()
        .border_color(cx.theme().colors.border)
        .cursor_pointer()
        .on_click(on_toggle)
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(Icon::new(collapse_icon).size(px(16.0)))
                .child(
                    div()
                        .text_sm()
                        .font_semibold()
                        .child("Packets by Protocol"),
                ),
        )
        .child(render_legend(categories, &colors, on_legend_click, cx));

    let content = if collapsed {
        div().into_any_element()
    } else {
        render_chart(buckets, tcp_color, udp_color, other_color, cx).into_any_element()
    };

    div()
        .w_full()
        .flex()
        .flex_col()
        .child(header)
        .child(content)
}

fn render_legend(
    categories: &[ProtocolCategory],
    colors: &[Hsla; 3],
    on_click: impl Fn(ProtocolCategory, &mut Window, &mut App) + 'static,
    cx: &App,
) -> impl IntoElement {
    let on_click = std::rc::Rc::new(on_click);

    div()
        .flex()
        .gap_4()
        .items_center()
        .children(categories.iter().enumerate().map(|(i, cat)| {
            let color = colors[i % colors.len()];
            let label = cat.label();
            let cat = *cat;
            let on_click = on_click.clone();

            div()
                .id(SharedString::from(format!("legend_{}", label)))
                .flex()
                .items_center()
                .gap_1()
                .cursor_pointer()
                .child(div().w_3().h_3().rounded_sm().bg(color))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(label),
                )
                .on_click(move |_event, window, cx| {
                    on_click(cat, window, cx);
                })
        }))
}

fn render_chart(
    buckets: Vec<HistogramBucket>,
    tcp_color: Hsla,
    udp_color: Hsla,
    other_color: Hsla,
    cx: &App,
) -> impl IntoElement {
    if buckets.is_empty() {
        return div()
            .flex()
            .items_center()
            .justify_center()
            .h(px(100.0))
            .text_sm()
            .text_color(cx.theme().muted_foreground)
            .child("No packet data")
            .into_any_element();
    }

    let buckets_for_chart = buckets.clone();

    let chart = StackedBarChart::new(buckets_for_chart, [tcp_color, udp_color, other_color])
        .tick_margin(5);

    let bg_color = cx.theme().background;
    let border_color = cx.theme().border;
    let text_color = cx.theme().foreground;
    let muted_color = cx.theme().muted_foreground;
    
    // Pre-calculate values needed for hover zones
    let bucket_count = buckets.len();

    // Clone buckets for the iterator
    let buckets_for_hover: Vec<_> = buckets.to_vec();

    div()
        .h(px(120.0))
        .w_full()
        .px_3()
        .py_2()
        .overflow_hidden()
        .relative()
        .child(chart)
        // Add a tooltip element for each bucket that shows on hover
        .children(buckets_for_hover.into_iter().enumerate().map(move |(index, bucket)| {
            let total = bucket.tcp + bucket.udp + bucket.other;
            let label = bucket.label.clone();
            let tcp = bucket.tcp;
            let udp = bucket.udp;
            let other = bucket.other;
            
            // Invisible hover zone for this bucket
            let zone_width = 100.0 / bucket_count as f32;
            let zone_left = index as f32 * zone_width;
            
            div()
                .id(SharedString::from(format!("bucket_hover_{}", index)))
                .group(SharedString::from(format!("bucket_group_{}", index)))
                .absolute()
                .top_0()
                .bottom(px(28.0))
                .left(relative(zone_left / 100.0))
                .w(relative(zone_width / 100.0))
                .cursor_crosshair()
                // Tooltip (hidden by default, shown on hover via group)
                .child(
                    div()
                        .invisible()
                        .group_hover(SharedString::from(format!("bucket_group_{}", index)), |this| this.visible())
                        .absolute()
                        .top(px(4.0))
                        .left(px(-60.0)) // Offset to roughly center the tooltip
                        .bg(bg_color)
                        .border_1()
                        .border_color(border_color)
                        .rounded_md()
                        .shadow_md()
                        .px_2()
                        .py_1()
                        .text_xs()
                        .child(
                            v_flex()
                                .gap_0p5()
                                .child(
                                    div()
                                        .font_semibold()
                                        .text_color(text_color)
                                        .child(label),
                                )
                                .child(
                                    h_flex()
                                        .gap_1()
                                        .child(div().w_2().h_2().rounded_sm().bg(tcp_color))
                                        .child(
                                            div()
                                                .text_color(muted_color)
                                                .child(format!("TCP: {:.0}", tcp)),
                                        ),
                                )
                                .child(
                                    h_flex()
                                        .gap_1()
                                        .child(div().w_2().h_2().rounded_sm().bg(udp_color))
                                        .child(
                                            div()
                                                .text_color(muted_color)
                                                .child(format!("UDP: {:.0}", udp)),
                                        ),
                                )
                                .child(
                                    h_flex()
                                        .gap_1()
                                        .child(div().w_2().h_2().rounded_sm().bg(other_color))
                                        .child(
                                            div()
                                                .text_color(muted_color)
                                                .child(format!("Other: {:.0}", other)),
                                        ),
                                )
                                .child(
                                    div()
                                        .font_semibold()
                                        .text_color(text_color)
                                        .child(format!("Total: {:.0}", total)),
                                ),
                        ),
                )
                // Vertical line indicator (hidden by default)
                .child(
                    div()
                        .invisible()
                        .group_hover(SharedString::from(format!("bucket_group_{}", index)), |this| this.visible())
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left_1_2()
                        .w(px(1.0))
                        .bg(border_color),
                )
        }))
        .into_any_element()
}
