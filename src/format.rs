use chrono::{DateTime, Local};

pub const SEPARATOR: &str = "^c#FFFFFF^|";

const GREEN: &str = "#00FF00"; // Low usage, Green
const YELLOW: &str = "#FFFF00"; // Moderate usage, Yellow
const ORANGE: &str = "#FFA500"; // High usage, Orange
const RED: &str = "#FF0000"; // Very high usage, Red

pub fn mem_format((used_ram, total_ram): (f32, f32)) -> String {
    let ram_percentage = used_ram / total_ram * 100.0;
    let color = if ram_percentage <= 25.0 {
        GREEN // Low usage, green
    } else if ram_percentage > 25.0 && ram_percentage <= 50.0 {
        YELLOW // Moderate usage, yellow
    } else if ram_percentage > 50.0 && ram_percentage <= 75.0 {
        ORANGE // High usage, orange
    } else {
        RED // Very high usage, red
    };
    format!(
        "^c{}^{:.2} {SEPARATOR} ^c{}^{:.2}",
        color, used_ram, color, total_ram
    )
}

pub fn cpu_load_avg_format(loadavg: f32) -> String {
    let (color, symbol) = if loadavg < 0.5 {
        (GREEN, "▁") // Low load, green
    } else if loadavg >= 0.5 && loadavg < 1.0 {
        (YELLOW, "■") // Moderate load, yellow
    } else if loadavg >= 1.0 && loadavg < 1.5 {
        (ORANGE, "▆") // High load, orange
    } else {
        (RED, "█") // Very high load, red
    };

    format!("^c{}^{} {:.2}", color, symbol, loadavg)
}

pub fn date_info_format(date: DateTime<Local>) -> String {
    format!("^c#2e95d3^{}", date.format("%b %d %a %I:%M %p").to_string())
}

pub fn network_io_format(rx_speed: f64, tx_speed: f64) -> String {
    format!(
        "^c{}^D {:>5.2} {SEPARATOR} ^c{}^U {:>5.2}",
        get_speed_color(rx_speed),
        rx_speed,
        get_speed_color(tx_speed),
        tx_speed
    )
}
fn get_speed_color(speed_mib: f64) -> String {
    if speed_mib < 1.0 {
        return GREEN.to_string(); // Green for speed < 1 MiB
    } else if speed_mib >= 1.0 && speed_mib < 10.0 {
        return YELLOW.to_string(); // Yellow for 1 MiB <= speed < 10 MiB
    } else {
        return RED.to_string(); // Red for speed >= 10 MiB
    }
}
/* instead of colors we can use bars, arrows or moon phases to show the strength */

// let bars = vec!["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
// let arrows = vec!["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"];
// let moon_phases = vec!["○", "◔", "◑", "◕", "●"];
// Assume signal strength range between -100 dBm and -30 dBm for this example
// let index =
//     ((signal_strength + 100) * (moon_phases.len() as i32 - 1) / 70) as usize;
// let index = index.min(moon_phases.len() - 1).max(0); // Clamp the index
// result
//     .push_str(format!("{} ({} dBm)", moon_phases[index], signal_strength).as_str())

pub fn wifi_info_format(strength: i32) -> String {
    let color = if strength > -70 {
        RED // Strong signal, green
    } else if strength <= -70 && strength > -80 {
        YELLOW // Moderate signal, yellow
    } else if strength <= -80 && strength > -90 {
        ORANGE // Weak signal, orange
    } else {
        RED // Very weak signal, red
    };
    format!("^c{}^{} dBm", color, strength)
}
