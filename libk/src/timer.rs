use core::arch::x86_64::{__cpuid, _mm_pause, _rdtsc};

#[inline]
fn calculate_tsc_frequency() -> u64 {
    let res = unsafe { __cpuid(0x15) };
    let (eax, ebx, ecx) = (res.eax as u64, res.ebx as u64, res.ecx as u64);
    if eax == 0 || ebx == 0 {
        return 0;
    }
    let core_freq = if ecx == 0 { 0u64 } else { ecx };
    (core_freq * ebx) + (eax >> 1) / eax
}

#[inline]
fn delay(delay: u64) {
    let ticks = unsafe { _rdtsc() } + delay;
    while unsafe { _rdtsc() } <= ticks {
        unsafe {
            _mm_pause();
        }
    }
}

/// Nearly identical to that of `core::time::Duration`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Duration {
    /// Second delay
    Secs(u64),
    /// Milliseconds delay
    Millis(u64),
    /// Microsecond delay
    Micros(u64),
    /// Nanosecond delay
    Nanos(u64),
}

/// Sleeps for the given number of microseconds or nanoseconds
pub fn sleep(time: Duration) {
    let f = calculate_tsc_frequency();
    let time = match time {
        Duration::Secs(d) => d * 1000000000 * f / 1000000000,
        Duration::Millis(d) => d * 1000000 * f / 1000000000,
        Duration::Micros(d) => d * 1000 * f / 1000000000,
        Duration::Nanos(d) => d * f / 1000000000,
    };
    delay(time);
}
