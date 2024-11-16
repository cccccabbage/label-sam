use image::DynamicImage;

use core::fmt;

pub struct UiState {
    pub img_label: String,
    pub img: Option<DynamicImage>, // TODO: change to epaint::ColorImage
    pub img_pos: Option<[f32; 2]>,

    pub prompt_type: PromptType,
    pub prompt_hover: PromptHover,

    pub boxes: Vec<[f32; 4]>,
    pub yolo_boxes: Vec<[f32; 4]>,
    pub points: Vec<[f32; 2]>,
    pub point_labels: Vec<f32>,

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
            img_pos: None,

            prompt_type: PromptType::None,
            prompt_hover: PromptHover::All,

            points: Vec::new(),
            point_labels: Vec::new(),
            yolo_boxes: Vec::new(),
            boxes: Vec::new(),

            drag_start: [-100.0, -100.0],
            drag_end: [-100.0, -100.0],
        }
    }

    pub fn add_point_label(&mut self, point: [f32; 2], label: f32) {
        self.points.push(point);
        self.point_labels.push(label);
    }

    pub fn add_box(&mut self, bbox: [f32; 4], is_manual: bool) {
        if is_manual {
            self.boxes.push(bbox);
        } else {
            self.yolo_boxes.push(bbox);
        }
    }

    pub fn add_yolo_boxes(&mut self, boxes: Vec<[f32; 4]>) {
        for bbox in boxes {
            self.add_box(bbox, false);
        }
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
