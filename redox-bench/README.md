# `redox-bench`

`redox-bench` is a performance auditing suite for Redox OS. It measures low-level kernel performance metrics, system IPC overhead, graphics rendering frame pacing, and network throughput under microkernel operating conditions.

## Benchmark Modules

- **`context-switch`**: Measures CPU task switching latency and register state save/restore overhead.
- **`network`**: Measures Redox scheme packet throughput and network stack latency.
- **`memory`**: Evaluates heap allocations, page mapping overhead, and physical memory residency.
- **`gpu`**: Audits user-space DRM graphics frame pacing and display buffer presentation rates.
- **`middleware`**: Evaluates scheme request wrapping and IPC microkernel context transition overhead.

## Installation & Usage

### Running Benchmarks

```bash
# Run all benchmark modules with default iterations (1000)
redox-bench all

# Run specific benchmark module with custom iteration count
redox-bench context-switch 50000

# Benchmark network scheme latency
redox-bench network 10000

# Audit memory allocation residency
redox-bench memory
```
