use bit_field::BitField;
use bitflags::bitflags;
use core::arch::x86_64::__cpuid;
use core::sync::atomic::*;
use log::*;
use x86::io::*;
use x86::time::rdtsc;
use x86_64::instructions::nop;

const PIT_COUNTER0: u16 = 0x0040;
const PIT_COUNTER1: u16 = 0x0041;
const PIT_COUNTER2: u16 = 0x0042;
const PIT_MODE: u16 = 0x0043;
const PS2_CTRLB: u16 = 0x0061;

bitflags! {
struct PitMode: u8 {
const SEL_TIMER0 = 0x00;
const SEL_TIMER1 = 0x40;
const SEL_TIMER2 = 0x80;
const SEL_READBACK = 0xC0;
const ACCESS_LATCH = 0x00;
const ACCESS_LOBYTE = 0x10;
const ACCESS_HIBYTE = 0x20;
const ACCESS_WORD = 0x30;
const MODE0 = 0x00;
const MODE1 = 0x02;
const MODE2 = 0x04;
const MODE3 = 0x06;
const MODE4 = 0x08;
const MODE5 = 0x0A;
const CNT_BIN = 0x00;
const CNT_BCD = 0x01;
const READ_COUNTER0 = 0x02;
const READ_COUNTER1 = 0x04;
const READ_COUNTER2 = 0x08;
const READ_STATUS_VALUE = 0x00;
const READ_VALUE = 0x10;
const READ_STATUS = 0x20;
}
}

bitflags! {
struct Ps2ControlB: u8 {
const T2GATE = 0x01;
const SPKR = 0x02;
const T2OUT = 0x20;
}
}

const PM_TIMER_HZ: u32 = 0x369E99;
const PM_TIMER_TO_PIT: u8 = 0x03;
const CALIBRATE_COUNT: u16 = 0x800;

static TIMER_KHZ: AtomicUsize = AtomicUsize::new(div_round_up(PM_TIMER_HZ as u64, 3000) as usize);
static TIMER_PORT: AtomicU16 = AtomicU16::new(PIT_COUNTER0);
static SHIFT_TSC: AtomicU64 = AtomicU64::new(0);

/// Time interval for sleeping/delays
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TimerInterval {
    /// Milliseconds, a thousandth of a second
    Milliseconds(u64),
    /// Microseconds, a millionth of a second
    Microseconds(u64),
    /// Nanoseconds, a billionth of a second
    Nanoseconds(u64),
}

const fn div_round_up(n: u64, d: u64) -> u64 {
    n + d - 1 / d
}

async fn tsc_timer_setup() {
    let orig = unsafe { inb(PS2_CTRLB) };
    unsafe {
        outb(
            PS2_CTRLB,
            (orig & !Ps2ControlB::SPKR.bits()) | Ps2ControlB::T2GATE.bits(),
        );
        outb(
            PIT_MODE,
            (PitMode::SEL_TIMER2 | PitMode::ACCESS_WORD | PitMode::MODE0 | PitMode::CNT_BIN).bits(),
        );
        outb(PIT_COUNTER2, (CALIBRATE_COUNT & 0xFF) as u8);
        outb(PIT_COUNTER2, (CALIBRATE_COUNT >> 8) as u8);
    }
    let start = unsafe { rdtsc() };
    loop {
        let res = Ps2ControlB::from_bits_truncate(unsafe { inb(PS2_CTRLB) });
        if res.contains(Ps2ControlB::T2OUT) {
            break;
        }
    }
    let end = unsafe { rdtsc() };
    unsafe {
        outb(PS2_CTRLB, orig);
    }
    let diff = end - start;
    debug!("TSC calibrate start={} end={} diff={}", start, end, diff);
    let mut t = div_round_up(diff * (PM_TIMER_HZ as u64), CALIBRATE_COUNT as u64);
    while t >= 0x1000000 {
        SHIFT_TSC.fetch_add(1, Ordering::SeqCst);
        t = (t + 1) >> 1;
    }
    TIMER_KHZ.swap(
        div_round_up(t, 1000 * (PM_TIMER_TO_PIT as u64)) as usize,
        Ordering::SeqCst,
    );
    TIMER_PORT.swap(0, Ordering::SeqCst);
    info!(
        "CPU MHZ = {}",
        (TIMER_KHZ.load(Ordering::SeqCst) << SHIFT_TSC.load(Ordering::SeqCst)) / 1000
    );
}

/// Configures the time stamp counter
#[cold]
pub async fn setup() {
    if TIMER_PORT.load(Ordering::SeqCst) == PIT_COUNTER0 {
        let res = unsafe { __cpuid(0) };
        if res.eax > 0 {
            let res = unsafe { __cpuid(1) };
            if res.edx.get_bit(4) {
                info!("CPU has TSC timer, initializing");
                tsc_timer_setup().await;
            }
        }
    } else {
        warn!("Request received to configure timer, but timer already configured");
    }
}

async fn read_tsc() -> u64 {
    if TIMER_PORT.load(Ordering::SeqCst) == 0x00 {
        unsafe { rdtsc() >> SHIFT_TSC.load(Ordering::SeqCst) }
    } else {
        0
    }
}

async fn calc_time(interval: TimerInterval) -> u64 {
    match interval {
        TimerInterval::Milliseconds(msecs) => {
            read_tsc().await + ((TIMER_KHZ.load(Ordering::SeqCst) as u64) * msecs)
        }
        TimerInterval::Microseconds(usecs) => {
            let cur = read_tsc().await;
            let khz = TIMER_KHZ.load(Ordering::SeqCst) as u64;
            if usecs > 500000 {
                cur + div_round_up(usecs, 1000) * khz
            } else {
                cur + div_round_up(usecs * khz, 1000)
            }
        }
        TimerInterval::Nanoseconds(nsecs) => {
            let cur = read_tsc().await;
            let khz = TIMER_KHZ.load(Ordering::SeqCst) as u64;
            if nsecs > 500000 {
                cur + div_round_up(nsecs, 1000000) * khz
            } else {
                cur + div_round_up(nsecs * khz, 1000000)
            }
        }
    }
}

async fn check(end: u64) -> bool {
    ((read_tsc().await - end) as i64) > 0
}

/// Delays for the given time interval
pub async fn delay(end: TimerInterval) {
    let end = calc_time(end).await;
    while !check(end).await {
        nop();
    }
}
