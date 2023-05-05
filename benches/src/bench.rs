use criterion::{criterion_group, criterion_main, Criterion};

use arp_scan::{vendor, vendor::Vendor};
use pnet::util::MacAddr;
use std::str::FromStr;

fn benchmark_load_bin(c: &mut Criterion) {
    c.bench_function("Load and parse ieee-oui BIN", |b| b.iter(|| {
        Vendor::new(Some("../data/ieee-oui.data"))
    }));
}

fn benchmark_load_csv(c: &mut Criterion) {
    c.bench_function("Load and parse ieee-oui CSV", |b| b.iter(|| {
        vendor::update(Some("../data/ieee-oui.csv"))
    }));
}

fn benchmark_find_mac(c: &mut Criterion) {
    let mut vendor = Vendor::new(None);
    c.bench_function("Find mac address from ieee-oui list", |b| b.iter(|| {
        let macs_to_find = vec![
        ];

        for mac in macs_to_find {
            let m = MacAddr::from_str(mac).unwrap();
            vendor.search_by_mac(&m);
        }
    }));
}

criterion_group!(benches, benchmark_load_bin, benchmark_load_csv);
criterion_main!(benches);
