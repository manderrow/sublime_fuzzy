#![feature(test)]
extern crate sublime_fuzzy;
extern crate test;

use std::hint::black_box;

use bumpalo::Bump;
use criterion::{Bencher, Criterion, criterion_group, criterion_main};

use sublime_fuzzy::{best_match, format_simple};

fn bench_group(c: &mut Criterion) {
    c.bench_function("empty", empty);
    dump_cache_stats();
    c.bench_function("short", short);
    dump_cache_stats();
    c.bench_function("url", url);
    dump_cache_stats();
    c.bench_function("url format", url_format);
    dump_cache_stats();
    c.bench_function("medium start", medium_start);
    dump_cache_stats();
    c.bench_function("medium_middle", medium_middle);
    dump_cache_stats();
    c.bench_function("medium_end", medium_end);
    dump_cache_stats();
    c.bench_function("long_start_close", long_start_close);
    dump_cache_stats();
    c.bench_function("long_middle_close", long_middle_close);
    dump_cache_stats();
}

fn dump_cache_stats() {
    /*println!(
        "  hits: {}",
        sublime_fuzzy::CACHE_HITS.swap(0, std::sync::atomic::Ordering::Relaxed)
    );
    println!(
        "misses: {}",
        sublime_fuzzy::CACHE_MISSES.swap(0, std::sync::atomic::Ordering::Relaxed)
    );*/
}

fn empty(b: &mut Bencher) {
    b.iter(|| 1);
}

fn short(b: &mut Bencher) {
    let mut bump = Bump::new();
    b.iter(|| {
        bump.reset();
        best_match(&bump, "jelly", "jellyfish");
    })
}

fn url(b: &mut Bencher) {
    let mut bump = Bump::new();
    b.iter(|| {
        bump.reset();
        black_box(best_match(&bump, "services", include_str!("services.txt")));
    });
}

fn url_format(b: &mut Bencher) {
    let mut bump = Bump::new();
    b.iter(|| {
        bump.reset();
        let t = include_str!("services.txt");

        format_simple(
            &best_match(&bump, "services", t).unwrap(),
            t,
            "<before>",
            "</after>",
        );
    })
}

fn medium_start(b: &mut Bencher) {
    let mut bump = Bump::new();
    b.iter(|| {
        bump.reset();
        black_box(best_match(&bump, "tracking", include_str!("tracking.txt")));
    });
}

fn medium_middle(b: &mut Bencher) {
    let mut bump = Bump::new();
    b.iter(|| {
        bump.reset();
        black_box(best_match(&bump, "requires", include_str!("tracking.txt")));
    });
}

fn medium_end(b: &mut Bencher) {
    let mut bump = Bump::new();
    b.iter(|| {
        bump.reset();
        black_box(best_match(&bump, "itself", include_str!("tracking.txt")));
    });
}

fn long_start_close(b: &mut Bencher) {
    let mut bump = Bump::new();
    b.iter(|| {
        bump.reset();
        black_box(best_match(
            &bump,
            "empty baseline",
            include_str!("empty-baseline.txt"),
        ));
    });
}

fn long_middle_close(b: &mut Bencher) {
    let mut bump = Bump::new();
    b.iter(|| {
        bump.reset();
        black_box(best_match(
            &bump,
            "rustc wound",
            include_str!("empty-baseline.txt"),
        ));
    });
}

criterion_group!(benches, bench_group);
criterion_main!(benches);
