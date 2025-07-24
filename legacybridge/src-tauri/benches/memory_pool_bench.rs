use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use legacybridge::conversion::{rtf_to_markdown, secure_rtf_to_markdown};
use legacybridge::conversion::pooled_converter::{rtf_to_markdown_pooled, secure_rtf_to_markdown_pooled, warm_up_pools, get_pool_stats};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// Custom allocator to track allocations
struct TrackingAllocator;

static ALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);
static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
        ALLOCATED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

fn reset_allocation_stats() {
    ALLOCATION_COUNT.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
}

fn get_allocation_stats() -> (usize, usize) {
    (
        ALLOCATION_COUNT.load(Ordering::Relaxed),
        ALLOCATED_BYTES.load(Ordering::Relaxed),
    )
}

fn generate_test_rtf(size: usize) -> String {
    let mut rtf = String::from(r"{\rtf1\ansi\deff0 {\fonttbl{\f0 Times New Roman;}}");
    
    for i in 0..size {
        rtf.push_str(&format!(r"\par Paragraph {} with some ", i));
        rtf.push_str(r"{\b bold text} and {\i italic text} ");
        rtf.push_str("and normal text. ");
    }
    
    rtf.push_str(r"\par}");
    rtf
}

fn benchmark_standard_conversion(c: &mut Criterion) {
    let small_rtf = generate_test_rtf(10);
    let medium_rtf = generate_test_rtf(100);
    let large_rtf = generate_test_rtf(1000);
    
    let mut group = c.benchmark_group("standard_conversion");
    
    for (name, rtf) in [("small", &small_rtf), ("medium", &medium_rtf), ("large", &large_rtf)] {
        group.bench_with_input(BenchmarkId::from_parameter(name), rtf, |b, rtf| {
            b.iter(|| {
                reset_allocation_stats();
                let _ = black_box(rtf_to_markdown(rtf));
                get_allocation_stats()
            });
        });
    }
    
    group.finish();
}

fn benchmark_pooled_conversion(c: &mut Criterion) {
    // Warm up pools before benchmarking
    warm_up_pools();
    
    let small_rtf = generate_test_rtf(10);
    let medium_rtf = generate_test_rtf(100);
    let large_rtf = generate_test_rtf(1000);
    
    let mut group = c.benchmark_group("pooled_conversion");
    
    for (name, rtf) in [("small", &small_rtf), ("medium", &medium_rtf), ("large", &large_rtf)] {
        group.bench_with_input(BenchmarkId::from_parameter(name), rtf, |b, rtf| {
            b.iter(|| {
                reset_allocation_stats();
                let _ = black_box(rtf_to_markdown_pooled(rtf));
                get_allocation_stats()
            });
        });
    }
    
    group.finish();
}

fn benchmark_allocation_comparison(c: &mut Criterion) {
    warm_up_pools();
    
    let test_rtf = generate_test_rtf(100);
    
    c.bench_function("allocation_overhead/standard", |b| {
        b.iter_custom(|iters| {
            let mut total_allocations = 0;
            let mut total_bytes = 0;
            
            for _ in 0..iters {
                reset_allocation_stats();
                let _ = black_box(rtf_to_markdown(&test_rtf));
                let (allocs, bytes) = get_allocation_stats();
                total_allocations += allocs;
                total_bytes += bytes;
            }
            
            std::time::Duration::from_nanos(total_allocations as u64)
        });
    });
    
    c.bench_function("allocation_overhead/pooled", |b| {
        b.iter_custom(|iters| {
            let mut total_allocations = 0;
            let mut total_bytes = 0;
            
            for _ in 0..iters {
                reset_allocation_stats();
                let _ = black_box(rtf_to_markdown_pooled(&test_rtf));
                let (allocs, bytes) = get_allocation_stats();
                total_allocations += allocs;
                total_bytes += bytes;
            }
            
            std::time::Duration::from_nanos(total_allocations as u64)
        });
    });
}

fn benchmark_pool_efficiency(c: &mut Criterion) {
    c.bench_function("pool_efficiency/cold_start", |b| {
        b.iter(|| {
            // Simulate cold start - no warmup
            let test_rtf = generate_test_rtf(50);
            reset_allocation_stats();
            let _ = black_box(rtf_to_markdown_pooled(&test_rtf));
            get_allocation_stats()
        });
    });
    
    c.bench_function("pool_efficiency/warm_start", |b| {
        warm_up_pools();
        b.iter(|| {
            let test_rtf = generate_test_rtf(50);
            reset_allocation_stats();
            let _ = black_box(rtf_to_markdown_pooled(&test_rtf));
            get_allocation_stats()
        });
    });
}

fn benchmark_memory_usage(c: &mut Criterion) {
    warm_up_pools();
    
    c.bench_function("memory_usage/pool_stats", |b| {
        b.iter(|| {
            let stats = get_pool_stats();
            black_box((
                stats.total_pooled_objects(),
                stats.estimated_memory_usage(),
            ))
        });
    });
}

criterion_group!(
    benches,
    benchmark_standard_conversion,
    benchmark_pooled_conversion,
    benchmark_allocation_comparison,
    benchmark_pool_efficiency,
    benchmark_memory_usage
);
criterion_main!(benches);