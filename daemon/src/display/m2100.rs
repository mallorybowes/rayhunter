//! Display support for the Inseego MiFi M2100 5G UW.
//!
//! The M2100 has a 320x240 framebuffer at `/dev/fb0`, but writes to it never
//! reach the panel: the vendor's Qt UI drives the display through a path that
//! issues no ioctl we can replicate, `FBIOPAN_DISPLAY` fails with EINVAL
//! (`yres_virtual == yres`, so there is no back buffer to flip), and neither
//! `write()` nor `mmap` + `msync` produces a visible change. See FINDINGS.md.
//!
//! What the device does have is a single user-controllable LED
//! (`/sys/class/leds/blue-led`, `max_brightness` 255), so this driver signals
//! state through it instead. That keeps the important property of the upstream
//! displays: a user who is not looking at the web UI still gets told when
//! something was detected.
//!
//! The device also has a PWM buzzer (`nvtl-buzzer`), which is sounded in time
//! with the Morse so an alert is noticeable without looking at the device.
//!
//!   Recording        - a brief heartbeat blink every few seconds, silent
//!   Paused           - dark, silent
//!   WarningDetected  - SOS in Morse (... --- ...) on both LED and buzzer,
//!                      repeating until cleared
//!
//! Two buzzer quirks, both confirmed on hardware:
//!   * Writing 1 to `control` returns EIO but *does* take effect - the buzzer
//!     sounds and `control` subsequently reads 1. The error is spurious.
//!   * Writing 0 to `control` returns EIO and does *not* take effect. Writing 0
//!     to `period` is the only reliable way to silence it.
use std::time::Duration;

use log::{info, warn};
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use rayhunter::analysis::analyzer::EventType;

use crate::config::{self, UiLevel};
use crate::display::DisplayState;

const LED: &str = "/sys/class/leds/blue-led";
const ON: &str = "255";
const OFF: &str = "0";

const BUZZER: &str = "/sys/devices/platform/soc:pwm_buzzer";

/// PWM period in nanoseconds, chosen per severity so the alert is
/// distinguishable by ear alone: higher pitch = more urgent.
/// 1_000_000 ns = 1 kHz, 500_000 = 2 kHz, 250_000 = 4 kHz.
fn buzzer_period_ns(event_type: EventType) -> &'static str {
    match event_type {
        EventType::Informational | EventType::Low => "1000000",
        EventType::Medium => "500000",
        EventType::High => "250000",
    }
}

/// Morse timing unit. A dot is one unit, a dash three, the gap between
/// elements one, and the gap between letters three.
const UNIT_MS: u64 = 180;

async fn write_led(file: &str, value: &str) {
    let path = format!("{LED}/{file}");
    if let Err(e) = tokio::fs::write(&path, value).await {
        warn!("failed writing {path}: {e}");
    }
}

/// Writes to the buzzer, tolerating the driver's spurious EIO on `control`
/// (see the quirks noted above). Anything else is worth a warning.
async fn write_buzzer(file: &str, value: &str) {
    let path = format!("{BUZZER}/{file}");
    if let Err(e) = tokio::fs::write(&path, value).await {
        if e.raw_os_error() != Some(5) {
            warn!("failed writing {path}: {e}");
        }
    }
}

async fn buzzer(on: bool, period_ns: &str) {
    if on {
        write_buzzer("period", period_ns).await;
        write_buzzer("control", "1").await;
    } else {
        // Writing control=0 returns EIO and leaves it sounding; period=0 works.
        write_buzzer("period", "0").await;
    }
}

/// A pattern is a list of (lit, duration) steps, played in a loop.
fn pattern_for(state: &DisplayState) -> Vec<(bool, u64)> {
    match state {
        DisplayState::Paused => vec![(false, 1000)],
        DisplayState::Recording => vec![(true, 60), (false, 3000)],
        DisplayState::WarningDetected { .. } => {
            let mut p = Vec::new();
            let mut letter = |counts: [u64; 3]| {
                for (i, units) in counts.iter().enumerate() {
                    p.push((true, units * UNIT_MS));
                    // one unit between elements, three between letters
                    p.push((false, if i == 2 { 3 * UNIT_MS } else { UNIT_MS }));
                }
            };
            letter([1, 1, 1]); // S
            letter([3, 3, 3]); // O
            letter([1, 1, 1]); // S
            p.push((false, 1200)); // pause before repeating
            p
        }
    }
}

pub fn update_ui(
    task_tracker: &TaskTracker,
    config: &config::Config,
    shutdown_token: CancellationToken,
    mut ui_update_rx: Receiver<DisplayState>,
) {
    if config.ui_level == UiLevel::Invisible {
        info!("Invisible mode, not spawning LED UI.");
        task_tracker.spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_token.cancelled() => break,
                    _ = ui_update_rx.recv() => {}
                }
            }
        });
        return;
    }

    info!("Starting M2100 LED UI on {LED}");
    task_tracker.spawn(async move {
        // Take the LED away from whatever trigger owns it (normally "timer").
        write_led("trigger", "none").await;

        let mut state = DisplayState::Recording;
        'outer: loop {
            let tone = match state {
                DisplayState::WarningDetected { event_type } => Some(buzzer_period_ns(event_type)),
                _ => None,
            };
            for (lit, ms) in pattern_for(&state) {
                write_led("brightness", if lit { ON } else { OFF }).await;
                if let Some(period) = tone {
                    buzzer(lit, period).await;
                }
                tokio::select! {
                    _ = shutdown_token.cancelled() => break 'outer,
                    next = ui_update_rx.recv() => {
                        match next {
                            // A new state interrupts the current pattern immediately.
                            Some(s) => { state = s; continue 'outer; }
                            None => break 'outer,
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_millis(ms)) => {}
                }
            }
        }

        // Hand the LED back to the vendor's trigger and silence the buzzer.
        buzzer(false, "0").await;
        write_led("brightness", OFF).await;
        write_led("trigger", "timer").await;
    });
}
