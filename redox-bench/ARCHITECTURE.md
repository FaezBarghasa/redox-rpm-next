# Architecture - redox-bench

`redox-bench` is designed as a modular benchmarking suite targeting specific microkernel subsystems and system call paths.

## Module Organization

```text
               ┌───────────────────────┐
               │    redox-bench CLI    │
               └───────────┬───────────┘
                           │
    ┌──────────────┬───────┼───────┬──────────────┐
    ▼              ▼       ▼       ▼              ▼
┌─────────┐   ┌─────────┐ ┌───┐ ┌──────────┐ ┌───────────┐
│ context │   │ network │ │mem│ │   gpu    │ │middleware │
│ switch  │   │         │ │   │ │          │ │  scheme   │
└─────────┘   └─────────┘ └───┘ └──────────┘ └───────────┘
```

## Subsystem Auditing Details

1. **Context Switch (`context_switch.rs`)**:
   - Triggers rapid task switching calls to evaluate CPU context save/restore latency.
2. **Network Benchmarks (`network.rs`)**:
   - Sends socket packets through the redox netstack scheme to measure throughput and latency.
3. **Memory Residency (`memory.rs`)**:
   - Measures heap allocation efficiency and page mapping penalties across memory operations.
4. **GPU Frame Pacing (`gpu.rs`)**:
   - Evaluates buffer swap latency and display synchronization in user-space DRM graphics stacks.
5. **Middleware Overhead (`middleware.rs`)**:
   - Evaluates wrapper latency for microkernel scheme messages (`redox-scheme`).
