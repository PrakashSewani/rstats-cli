use rstats::collector::{Collector, SysinfoCollector};

#[test]
fn sysinfo_collector_produces_a_snapshot() {
    let mut collector = SysinfoCollector::new();
    collector.warm_up();
    let snapshot = collector.collect().expect("system collection should succeed");
    assert!(snapshot.cpu.total_usage.is_finite());
    assert!(
        snapshot.memory.used_bytes <= snapshot.memory.total_bytes
            || snapshot.memory.total_bytes == 0
    );
    assert!(snapshot
        .disks
        .iter()
        .all(|disk| disk.used_percent.is_finite() && (0.0..=100.0).contains(&disk.used_percent)));
}
