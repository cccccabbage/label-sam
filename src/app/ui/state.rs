use image::DynamicImage;

use std::sync::Arc;

const CHECKBOX_NAMES: [&str; 1] = ["Point Prompts"];

pub struct UiState {
    pub img_label: String,
    pub img: Option<Arc<DynamicImage>>,

    pub checkbox_states: [bool; CHECKBOX_NAMES.len()],
    pub checkbox_names: [&'static str; CHECKBOX_NAMES.len()],
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            img_label: "Load image first".to_string(),
            img: None,

            checkbox_states: [false; CHECKBOX_NAMES.len()],
            checkbox_names: CHECKBOX_NAMES,
        }
    }
}
