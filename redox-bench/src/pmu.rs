use std::fs::File;
use std::io::{self, Read};

pub struct Pmu;

impl Pmu {
    /// Read the cycle counter using the sys:pmu scheme.
    /// Fallback to rdtsc if the scheme is unavailable and on x86_64.
    pub fn read_cycles() -> u64 {
        if let Ok(mut file) = File::open("sys:pmu") {
            let mut buf = String::new();
            if file.read_to_string(&mut buf).is_ok() {
                if let Some(line) = buf.lines().next() {
                    if let Some(val_str) = line.strip_prefix("tsc:") {
                        if let Ok(val) = val_str.parse::<u64>() {
                            return val;
                        }
                    }
                }
            }
        }

        #[cfg(target_arch = "x86_64")]
        {
            unsafe { std::arch::x86_64::_rdtsc() }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            0
        }
    }

    /// Measure the duration of a closure in cycles.
    pub fn measure<F: FnOnce()>(f: F) -> u64 {
        let start = Self::read_cycles();
        f();
        let end = Self::read_cycles();
        end.saturating_sub(start)
    }
}
