use crate::pmu::Pmu;
use std::sync::mpsc;
use std::thread;

pub fn benchmark_context_switch(iterations: u64) {
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    let t1 = thread::spawn(move || {
        for _ in 0..iterations {
            let _ = rx1.recv();
            let _ = tx2.send(());
        }
    });

    let start = Pmu::read_cycles();
    for _ in 0..iterations {
        let _ = tx1.send(());
        let _ = rx2.recv();
    }
    let end = Pmu::read_cycles();

    t1.join().unwrap();

    let total_cycles = end.saturating_sub(start);
    let avg_cycles = total_cycles / (iterations * 2); // Two context switches per iteration

    // Assuming 2.0 GHz for rough µs estimation if cycles are used.
    // In a real system, we'd read the frequency from sys:cpu or similar.
    println!(
        "[BENCH] context_switch: {} iters | avg_cycles: {}",
        iterations, avg_cycles
    );
}
