use image::DynamicImage;

use core::fmt;
use std::sync::Arc;

pub struct UiState {
    pub img_label: String,
    pub img: Option<Arc<DynamicImage>>, // TODO: change to epaint::ColorImage

    pub prompt_type: PromptType,
    pub prompt_hover: PromptHover,

    pub prompt_points: Vec<[f32; 2]>,
    pub prompt_boxes: Vec<[f32; 4]>,

    pub drag_start: [f32; 2],
    pub drag_end: [f32; 2],
}

#[derive(PartialEq, strum_macros::EnumIter, Copy, Clone)]
pub enum PromptType {
    None,
    Point,
    Box,
}

#[derive(PartialEq, strum_macros::EnumIter, Copy, Clone)]
pub enum PromptHover {
    None,
    Point,
    Box,
    All,
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            img_label: "Load image first".to_string(),
            img: None,

            prompt_type: PromptType::None,
            prompt_hover: PromptHover::All,

            prompt_points: Vec::new(),
            prompt_boxes: Vec::new(),

            drag_start: [-100.0, -100.0],
            drag_end: [-100.0, -100.0],
        }
    }

    pub fn add_point(&mut self, p: impl Into<[f32; 2]>) {
        self.prompt_points.push(p.into());
    }

    pub fn add_box(&mut self, bb: impl Into<[f32; 4]>) {
        self.prompt_boxes.push(bb.into());
    }
}

impl fmt::Display for PromptType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PromptType::None => write!(f, "None"),
            PromptType::Point => write!(f, "Point"),
            PromptType::Box => write!(f, "Box"),
        }
    }
}

impl fmt::Display for PromptHover {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PromptHover::None => write!(f, "None"),
            PromptHover::Point => write!(f, "Point"),
            PromptHover::Box => write!(f, "Box"),
            PromptHover::All => write!(f, "All"),
        }
    }
}
