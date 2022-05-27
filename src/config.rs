use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Config {
	pub refresh_rate_ms: u16,
	pub image: Image,
	pub metadata: Metadata,
	pub controls_bar: ControlsBar,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Image {
	pub force_symbols: bool,
	pub margins: Box,
	pub size: (u16, u16),
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Metadata {
	pub vertical_margins: (u16, u16),  // Top, bottom
	pub interline_gap: u16,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct ControlsBar {
	pub button_prev: Button,
	pub button_playpause: ButtonSwitchable,
	pub button_next: Button,
	pub is_background_present: bool,
	pub are_caps_present: bool,
	pub cap_left: char,
	pub cap_right: char,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Button {
	pub icon: char,
	pub padding: (u16, u16),  // Additional clickable area around button
	pub margins: (u16, u16),  // Additional non-clickable area around button
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct ButtonSwitchable {
	pub icon_state1: char,    // Playing
	pub icon_state2: char,    // Paused
	pub padding: (u16, u16),  // Additional clickable area around button
	pub margins: (u16, u16),  // Additional non-clickable area around button
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Box {
	pub top: u16,
	pub bottom: u16,
	pub left: u16,
	pub right: u16,
}