#![feature(test)]
extern crate sublime_fuzzy;
extern crate test;

use std::hint::black_box;

use bumpalo::Bump;
use criterion::{Bencher, Criterion, criterion_group, criterion_main};

use sublime_fuzzy::{Query, Scoring, best_match, format_simple};

fn bench_group(c: &mut Criterion) {
    c.bench_function("empty", empty);
    c.bench_function("short", short);
    c.bench_function("url", url);
    c.bench_function("medium start", medium_start);
    c.bench_function("medium_middle", medium_middle);
    c.bench_function("medium_end", medium_end);
    c.bench_function("long_start_close", long_start_close);
    c.bench_function("long_middle_close", long_middle_close);

    // c.bench_function("empty cached query", empty_cached_query);
    // c.bench_function("short cached query", short_cached_query);
    // c.bench_function("url cached query", url_cached_query);
    // c.bench_function("medium  cached querystart", medium_sta_cached_queryrt);
    // c.bench_function("medium_middle cached query", medium_middle_cached_query);
    // c.bench_function("medium_end cached query", medium_end_cached_query);
    c.bench_function(
        "long_start_close cached query",
        long_start_close_cached_query,
    );
    c.bench_function(
        "long_middle_close cached query",
        long_middle_close_cached_query,
    );

    c.bench_function("url format", url_format);
}

fn empty(b: &mut Bencher) {
    b.iter(|| 1);
}

fn short(b: &mut Bencher) {
    let mut bump = Bump::new();
    b.iter(|| {
        bump.reset();
        best_match(
            &bump,
            &Query::new(&bump, "jelly"),
            &Scoring::default(),
            "jellyfish",
        );
    })
}

fn url(b: &mut Bencher) {
    let mut bump = Bump::new();
    b.iter(|| {
        bump.reset();
        black_box(best_match(
            &bump,
            &Query::new(&bump, "services"),
            &Scoring::default(),
            include_str!("services.txt"),
        ));
    });
}

fn medium_start(b: &mut Bencher) {
    let mut bump = Bump::new();
    b.iter(|| {
        bump.reset();
        black_box(best_match(
            &bump,
            &Query::new(&bump, "tracking"),
            &Scoring::default(),
            include_str!("tracking.txt"),
        ));
    });
}

fn medium_middle(b: &mut Bencher) {
    let mut bump = Bump::new();
    b.iter(|| {
        bump.reset();
        black_box(best_match(
            &bump,
            &Query::new(&bump, "requires"),
            &Scoring::default(),
            include_str!("tracking.txt"),
        ));
    });
}

fn medium_end(b: &mut Bencher) {
    let mut bump = Bump::new();
    b.iter(|| {
        bump.reset();
        black_box(best_match(
            &bump,
            &Query::new(&bump, "itself"),
            &Scoring::default(),
            include_str!("tracking.txt"),
        ));
    });
}

fn long_start_close(b: &mut Bencher) {
    let mut bump = Bump::new();
    b.iter(|| {
        bump.reset();
        black_box(best_match(
            &bump,
            &Query::new(&bump, "empty baseline"),
            &Scoring::default(),
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
            &Query::new(&bump, "rustc wound"),
            &Scoring::default(),
            include_str!("empty-baseline.txt"),
        ));
    });
}

fn long_start_close_cached_query(b: &mut Bencher) {
    let bump = Bump::new();
    let query = Query::new(&bump, "empty baseline");
    let mut bump = Bump::new();
    b.iter(|| {
        bump.reset();
        black_box(best_match(
            &bump,
            &query,
            &Scoring::default(),
            include_str!("empty-baseline.txt"),
        ));
    });
}

fn long_middle_close_cached_query(b: &mut Bencher) {
    let bump = Bump::new();
    let query = Query::new(&bump, "rustc wound");
    let mut bump = Bump::new();
    b.iter(|| {
        bump.reset();
        black_box(best_match(
            &bump,
            &query,
            &Scoring::default(),
            include_str!("empty-baseline.txt"),
        ));
    });
}

fn url_format(b: &mut Bencher) {
    let mut bump = Bump::new();
    b.iter(|| {
        bump.reset();
        let t = include_str!("services.txt");

        format_simple(
            &best_match(
                &bump,
                &Query::new(&bump, "services"),
                &Scoring::default(),
                t,
            )
            .unwrap(),
            t,
            "<before>",
            "</after>",
        );
    })
}

criterion_group!(benches, bench_group);
criterion_main!(benches);
