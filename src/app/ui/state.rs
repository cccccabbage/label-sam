use crate::app::model::sam::prompt::Prompt;

use super::instance::Instance;

use image::DynamicImage;

use core::fmt;
use std::path::PathBuf;

pub struct UiState {
    pub img_label: String,
    pub img_pos: Option<[f32; 2]>,

    pub img: Option<DynamicImage>,
    pub img_ori_size: Option<[f32; 2]>,
    pub img_file_size: Option<f32>,
    pub img_path: Option<PathBuf>,

    pub prompt_type: PromptType,
    pub prompt_hover: PromptHover,

    pub selection_mode: bool,

    pub instances: Vec<Instance>,
    pub selection: Vec<bool>,
    pub select_all: bool,

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
            img_pos: None,

            img: None,
            img_path: None,
            img_ori_size: None,
            img_file_size: None,

            prompt_type: PromptType::None,
            prompt_hover: PromptHover::All,

            selection_mode: false,
            select_all: false,

            instances: Vec::new(),
            selection: Vec::new(),

            drag_start: [-100.0, -100.0],
            drag_end: [-100.0, -100.0],
        }
    }

    fn add_instance(&mut self, instance: Instance) {
        self.instances.push(instance);
        self.selection.push(true);
    }

    // if selecton < 0, add a new instance
    // else update the selected one
    pub fn add_point_label(&mut self, point: [f32; 2], label: f32) {
        let selection = self.check_selection();
        if selection < 0 {
            let mut instance = Instance::new();
            instance.add_point_label(point[0], point[1], label);
            self.add_instance(instance);
        } else {
            self.instances[selection as usize].add_point_label(point[0], point[1], label);
        }
    }

    // if selecton < 0, add a new instance
    // else update the selected one
    pub fn add_box(&mut self, bbox: [f32; 4], is_manual: bool) {
        let selection = if is_manual {
            self.check_selection()
        } else {
            -1
        };

        if selection < 0 {
            let mut instance = Instance::new();
            instance.add_box(bbox, is_manual);
            self.add_instance(instance);
        } else {
            self.instances[selection as usize].add_box(bbox, is_manual);
        }
    }

    pub fn add_yolo_boxes(&mut self, boxes: Vec<[f32; 4]>) {
        for bbox in boxes {
            self.add_box(bbox, false);
        }
    }

    pub fn draw_prompts(&self, painter: &egui::Painter) {
        assert_eq!(self.instances.len(), self.selection.len());
        for (s, ins) in self.selection.iter().zip(self.instances.iter()) {
            if *s {
                ins.draw_prompt(
                    painter,
                    &self.prompt_hover,
                    self.img_ori_size.as_ref().unwrap(),
                    self.img_pos.as_ref().unwrap(),
                );
            }
        }
    }

    pub fn draw_outline(&self, painter: &egui::Painter) {
        assert_eq!(self.instances.len(), self.selection.len());
        for (s, ins) in self.selection.iter().zip(self.instances.iter()) {
            if *s {
                ins.draw_outline(
                    painter,
                    self.img_ori_size.as_ref().unwrap(),
                    self.img_pos.as_ref().unwrap(),
                );
            }
        }
    }

    pub fn format_prompts(&self) -> Vec<Vec<Prompt>> {
        let prompts = self
            .instances
            .iter()
            .map(|ins| ins.prompts.clone())
            .collect();

        prompts
    }

    pub fn check_selection(&self) -> i32 {
        // go through the selection twice
        // Considering that the Vec is not big at all, this is fine.

        if self.select_all {
            return -1;
        }

        let count = self.selection.iter().filter(|&&x| x).count();
        if count > 1 || count == 0 {
            -1
        } else {
            match self.selection.iter().position(|&x| x) {
                None => -1,
                Some(i) => i as i32,
            }
        }
    }

    pub fn change_select_all(&mut self) {
        for s in self.selection.iter_mut() {
            *s = self.select_all;
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
