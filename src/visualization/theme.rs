//! Theme constants for visualization
//!
//! Color palette matches Discord's dark theme and VRChat status page.

use plotters::style::RGBColor;

/// Chart background color (GitHub dark)
pub const BG_COLOR: RGBColor = RGBColor(0x0D, 0x11, 0x17);

/// Primary graph color
pub const GRAPH_COLOR: RGBColor = RGBColor(0x6C, 0x62, 0x84);

/// Light text color for titles and labels
pub const TEXT_COLOR: RGBColor = RGBColor(0xF2, 0xF3, 0xF5);

/// Muted text color for secondary elements
pub const MUTED_COLOR: RGBColor = RGBColor(0x94, 0x9B, 0xA4);

/// Status colors
pub const GREEN: RGBColor = RGBColor(0x57, 0xF2, 0x87);
pub const RED: RGBColor = RGBColor(0xED, 0x42, 0x45);
#[allow(dead_code)]
pub const YELLOW: RGBColor = RGBColor(0xFE, 0xE7, 0x5C);
#[allow(dead_code)]
pub const ORANGE: RGBColor = RGBColor(0xF0, 0xB1, 0x32);

/// Dashboard configuration
pub const IMAGE_SIZE: u32 = 2400;
pub const HOURS_RANGE: i64 = 12;
pub const DOWNSAMPLE_MINUTES: i64 = 5;

/// Font sizes (scaled for high resolution)
pub const TITLE_FONT_SIZE: u32 = 48;
pub const LABEL_FONT_SIZE: u32 = 22;
