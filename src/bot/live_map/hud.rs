/// Text and bar data layered over the map, assembled by the view so this
/// module stays free of statistics/settings types.
#[derive(Clone, Debug, Default)]
pub struct MapHud {
    /// Large centered text — the map name, or `Loading`; empty skips it.
    pub title: String,
    /// `RUNNING`/`WAITING` plus session time, drawn under the title.
    pub status: String,
    /// Left-edge statistics column (`cre/h`, `uri/h`, `exp/h`, `hon/h`,
    /// `cargo`, `death`).
    pub stats: Vec<String>,
    /// Hero configuration label (`1C`/`2C`) at the bottom-left.
    pub config: String,
    /// Hero name + health/shield bar at the bottom-left.
    pub hero: Option<BarHud>,
    /// Locked-target bar at the bottom-right, `Some` while a ship is
    /// selected.
    pub target: Option<BarHud>,
    /// Entity id of the locked target; its marker is painted filled.
    pub target_id: Option<i32>,
    /// Top-left `<reloads> - <since last>/<interval>` line; empty skips it
    /// and the refresh icon.
    pub refresh_info: String,
    /// Module status lines under the refresh counter (top-left).
    pub module_status: Vec<String>,
    /// Top-right text after `SID: `; empty skips it.
    pub sid: String,
    /// Right-edge booster lines: text and color, in display order.
    pub boosters: Vec<(String, u32)>,
}

/// One DarkBot-style name + health/shield bar pair.
#[derive(Clone, Debug)]
pub struct BarHud {
    pub name: String,
    pub details: String,
    /// Enemy-style red name vs the hero's neutral one.
    pub enemy: bool,
    pub hp: f32,
    pub max_hp: f32,
    /// Whether `hp` already includes the nanohull pool.
    pub hp_includes_nano: bool,
    pub nano: f32,
    pub nano_max: f32,
    pub shield: f32,
    pub max_shield: f32,
}

/// `###,###,###` — DarkBot's stat format, space-grouped like its UI.
pub fn format_amount(value: f64) -> String {
    let negative = value < 0.0;
    let digits = value.abs().round() as u64;
    let mut grouped = String::new();
    for (index, digit) in digits.to_string().chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push(' ');
        }
        grouped.push(digit);
    }
    let mut text: String = grouped.chars().rev().collect();
    if negative {
        text.insert(0, '-');
    }
    text
}
