use super::{Endpoint, Flow, IPAddress, Protocol};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FlowFilter<'a> {
    needle: String,
    timestamp_origin: Option<f64>,
    prefer_names: bool,
    name_resolutions: Option<&'a HashMap<IPAddress, Vec<String>>>,
}

impl<'a> FlowFilter<'a> {
    pub fn new(
        query: impl AsRef<str>,
        timestamp_origin: Option<f64>,
        prefer_names: bool,
        name_resolutions: Option<&'a HashMap<IPAddress, Vec<String>>>,
    ) -> Self {
        let needle = query.as_ref().trim().to_lowercase();
        Self {
            needle,
            timestamp_origin,
            prefer_names,
            name_resolutions,
        }
    }

    pub fn matches_flow(&self, flow: &Flow) -> bool {
        if self.is_match_all() {
            return true;
        }

        let timestamp = FlowFormatter::timestamp(flow.timestamp, self.timestamp_origin);
        if self.matches(&timestamp) {
            return true;
        }

        let src_ip =
            FlowFormatter::ip_address(&flow.source.ip, self.prefer_names, self.name_resolutions);
        if self.matches(&src_ip) {
            return true;
        }

        let src_endpoint =
            FlowFormatter::endpoint(&flow.source, self.prefer_names, self.name_resolutions);
        if self.matches(&src_endpoint) {
            return true;
        }

        if self.matches(&flow.source.port.to_string()) {
            return true;
        }

        let dst_ip = FlowFormatter::ip_address(
            &flow.destination.ip,
            self.prefer_names,
            self.name_resolutions,
        );
        if self.matches(&dst_ip) {
            return true;
        }

        let dst_endpoint =
            FlowFormatter::endpoint(&flow.destination, self.prefer_names, self.name_resolutions);
        if self.matches(&dst_endpoint) {
            return true;
        }

        if self.matches(&flow.destination.port.to_string()) {
            return true;
        }

        let protocol = FlowFormatter::protocol(&flow.protocol);
        self.matches(&protocol)
    }

    pub fn is_match_all(&self) -> bool {
        self.needle.is_empty()
    }

    pub fn timestamp_origin(&self) -> Option<f64> {
        self.timestamp_origin
    }

    fn matches(&self, value: &str) -> bool {
        value.to_lowercase().contains(&self.needle)
    }
}

pub struct FlowFormatter;

impl FlowFormatter {
    pub fn timestamp(timestamp: f64, origin: Option<f64>) -> String {
        let relative = origin.map(|start| timestamp - start).unwrap_or(timestamp);
        format!("{:.6}", relative)
    }

    pub fn ip_address(
        ip: &IPAddress,
        prefer_names: bool,
        name_resolutions: Option<&HashMap<IPAddress, Vec<String>>>,
    ) -> String {
        if prefer_names
            && let Some(first) = name_resolutions
                .and_then(|m| m.get(ip))
                .and_then(|names| names.first())
        {
            return first.clone();
        }
        ip.to_string()
    }

    pub fn endpoint(
        endpoint: &Endpoint,
        prefer_names: bool,
        name_resolutions: Option<&HashMap<IPAddress, Vec<String>>>,
    ) -> String {
        let ip = Self::ip_address(&endpoint.ip, prefer_names, name_resolutions);
        format!("{}:{}", ip, endpoint.port)
    }

    pub fn protocol(protocol: &Protocol) -> String {
        match protocol {
            Protocol::TCP => "TCP".to_string(),
            Protocol::UDP => "UDP".to_string(),
            Protocol::Other(n) => format!("Proto-{}", n),
        }
    }

    pub fn port(port: u16) -> String {
        port.to_string()
    }
}
