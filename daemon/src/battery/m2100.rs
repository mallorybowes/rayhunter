use std::path::Path;

use crate::{
    battery::{BatteryState, get_level_from_percentage_file, is_plugged_in_from_file},
    error::RayhunterError,
};

const BATTERY_LEVEL_FILE: &str = "/sys/class/power_supply/battery/capacity";

/// Supplies that can indicate external power, in the order they are checked.
///
/// Unlike devices with a single USB supply, this one splits them: charging from
/// a computer's data port registers on `pc_port`, a wall charger on `usb`, and
/// `dc` covers a barrel-jack style input. Measured on hardware while plugged
/// into a laptop, `pc_port/online` reads 1 while `usb/online` reads 0 — so
/// checking only the obvious `usb` path would report a charging device as
/// running on battery.
const PLUGGED_IN_STATE_FILES: [&str; 3] = [
    "/sys/class/power_supply/pc_port/online",
    "/sys/class/power_supply/usb/online",
    "/sys/class/power_supply/dc/online",
];

pub async fn get_battery_state() -> Result<BatteryState, RayhunterError> {
    let level = get_level_from_percentage_file(Path::new(BATTERY_LEVEL_FILE)).await?;

    // Any supply reporting online counts as plugged in. A supply that cannot be
    // read is treated as offline rather than fatal: the device exposes several
    // and only some are populated on a given unit.
    let mut is_plugged_in = false;
    for path in PLUGGED_IN_STATE_FILES {
        if let Ok(true) = is_plugged_in_from_file(Path::new(path)).await {
            is_plugged_in = true;
            break;
        }
    }

    Ok(BatteryState {
        level,
        is_plugged_in,
    })
}
