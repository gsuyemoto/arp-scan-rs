use criterion::{criterion_group, criterion_main, Criterion};

use arp_scan::{vendor, vendor::Vendor};
use pnet::util::MacAddr;
use std::str::FromStr;
use regex::{Captures, Regex};

fn benchmark_load_bin(c: &mut Criterion) {
    c.bench_function("Load and parse ieee-oui BIN", |b| b.iter(|| {
        Vendor::new()
    }));
}

fn benchmark_load_csv(c: &mut Criterion) {
    c.bench_function("Load and parse ieee-oui CSV", |b| b.iter(|| {
        vendor::update()
    }));
}

fn benchmark_find_mac(c: &mut Criterion) {
    let mut num_macs: usize = 100;
    let mut vend = Vendor::new();
    if vend.records.len() < num_macs {num_macs = vend.records.len()}

    // Go and load list of OUIs from file
    // extract first n (num_macs) OUI MAC addresses
    // and create n (num_macs) number of fake MACS
    // to search for (e.g. AA:BB:CC:00:00:00) basically
    // taking real OUI and appending 00:00:00
    let mut macs_to_find = Vec::new();
    let mut current_count: usize = 0;
    for oui in vend.records.keys() {
        current_count += 1;
        if current_count > num_macs {break}

        let re = Regex::new(r"([[:xdigit:]]{2})([[:xdigit:]]{2})([[:xdigit:]]{2})")
            .expect("Oops! Something is wrong with regex expression.");

        let hex_format = re.replace(oui, |caps: &Captures| {
            format!("{}:{}:{}:00:00:00", &caps[1], &caps[2], &caps[3])
        });
        let fake_mac = MacAddr::from_str(hex_format.as_ref());

        match fake_mac {
            Ok(mac) => macs_to_find.push(mac),
            Err(e)  => eprintln!("Error converting mac string to MacAddr: {:#?}", e),  
        }
    }

    c.bench_function("Find mac address from ieee-oui list", |b| b.iter(|| {
        for mac in &macs_to_find {
            vend.search_by_mac(&mac);
        }
    }));
}

criterion_group!(benches, benchmark_load_bin, benchmark_load_csv, benchmark_find_mac);
criterion_main!(benches);
