use super::{FlowLoadController, FlowLoadStatus};
use std::fs::File;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn controller_reports_error_for_invalid_pcap() {
    let tmp_path = std::env::temp_dir().join("wirecrab_empty_test.pcapng");
    // Create an empty file to force the parser to error.
    File::create(&tmp_path).expect("create temp pcap");

    let mut controller = FlowLoadController::new(tmp_path.clone());

    // Poll until the background thread reports an error or time out.
    let mut saw_error = false;
    for _ in 0..100 {
        match controller.poll() {
            FlowLoadStatus::Error(msg) => {
                saw_error = true;
                assert!(msg.contains("Failed") || msg.contains("error"));
                break;
            }
            FlowLoadStatus::Loading { .. } => sleep(Duration::from_millis(10)),
            FlowLoadStatus::Ready { .. } => panic!("unexpected success for empty pcap"),
            FlowLoadStatus::Idle => {}
        }
    }

    assert!(saw_error, "loader should report error for invalid pcap");

    // After an error, subsequent polls should return Idle.
    if let FlowLoadStatus::Idle = controller.poll() {
    } else {
        panic!("controller should be idle after error");
    }

    let _ = std::fs::remove_file(tmp_path);
}

#[test]
fn controller_loads_valid_pcap() {
    let path = std::path::PathBuf::from("testdata/valid_tcp_udp.pcapng");
    assert!(path.exists(), "expected testdata/valid_tcp_udp.pcapng to exist");

    let mut controller = FlowLoadController::new(path);

    let mut got_ready = false;
    let mut flows_seen = 0usize;
    let mut saw_timestamp = false;
    for _ in 0..200 {
        match controller.poll() {
            FlowLoadStatus::Ready { flows, start_timestamp, name_resolutions } => {
                got_ready = true;
                flows_seen = flows.len();
                saw_timestamp = start_timestamp.is_some();
                let _ = name_resolutions.len();
                break;
            }
            FlowLoadStatus::Loading { .. } => sleep(Duration::from_millis(10)),
            FlowLoadStatus::Error(msg) => panic!("unexpected error: {msg}"),
            FlowLoadStatus::Idle => {}
        }
    }

    assert!(got_ready, "loader did not finish in time");
    assert!(flows_seen > 0, "expected at least one flow from tcp/udp capture");
    assert!(saw_timestamp, "expected start timestamp from capture");
}

#[test]
fn controller_reports_progress_before_completion() {
    let path = std::path::PathBuf::from("testdata/randpkt_mixed.pcapng");
    assert!(path.exists(), "expected testdata/randpkt_mixed.pcapng to exist");

    let mut controller = FlowLoadController::new(path);

    let mut saw_progress = false;
    let mut finished = false;
    for _ in 0..500 {
        match controller.poll() {
            FlowLoadStatus::Loading { progress } => {
                saw_progress = true;
                if progress >= 1.0 {
                    // allow completion soon after
                }
            }
            FlowLoadStatus::Ready { .. } => {
                finished = true;
                break;
            }
            FlowLoadStatus::Error(msg) => panic!("unexpected error: {msg}"),
            FlowLoadStatus::Idle => {}
        }
        sleep(Duration::from_millis(5));
    }

    assert!(saw_progress, "expected at least one progress update");
    assert!(finished, "loader did not finish in time");

    // After completion, subsequent polls should be idle.
    assert!(matches!(controller.poll(), FlowLoadStatus::Idle));
}
