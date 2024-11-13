use image::DynamicImage;

use core::fmt;
use std::sync::Arc;

pub struct UiState {
    pub img_label: String,
    pub img: Option<Arc<DynamicImage>>,

    pub prompt_type: PromptType,
}

#[derive(PartialEq, strum_macros::EnumIter, Copy, Clone)]
pub enum PromptType {
    Void,
    Point,
    Box,
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            img_label: "Load image first".to_string(),
            img: None,

            prompt_type: PromptType::Void,
        }
    }
}

impl fmt::Display for PromptType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PromptType::Void => write!(f, "None"),
            PromptType::Point => write!(f, "Point"),
            PromptType::Box => write!(f, "Box"),
        }
    }
}
