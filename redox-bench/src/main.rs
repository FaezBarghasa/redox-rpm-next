mod context_switch;
mod gpu;
mod memory;
mod middleware;
mod network;
mod pmu;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: redox-bench <benchmark> [iterations]");
        println!("Available benchmarks: all, context-switch, network, memory, gpu, middleware");
        return;
    }

    let benchmark = &args[1];
    let iterations = args
        .get(2)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(1000);

    match benchmark.as_str() {
        "all" => {
            context_switch::benchmark_context_switch(iterations);
            network::benchmark_network(iterations);
            memory::benchmark_memory_residency();
            gpu::benchmark_gpu_pacing(iterations);
            middleware::benchmark_middleware_overhead();
        }
        "context-switch" => context_switch::benchmark_context_switch(iterations),
        "network" => network::benchmark_network(iterations),
        "memory" => memory::benchmark_memory_residency(),
        "gpu" => gpu::benchmark_gpu_pacing(iterations),
        "middleware" => middleware::benchmark_middleware_overhead(),
        _ => println!("Unknown benchmark: {}", benchmark),
    }
}
