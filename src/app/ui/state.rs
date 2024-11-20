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
    pub operation_mode: OptMode,

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

#[derive(PartialEq, strum_macros::EnumIter, Copy, Clone)]
pub enum OptMode {
    None,
    NewInstance,
    AddOn,
    Delete,
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
            operation_mode: OptMode::None,

            selection_mode: false,

            instances: Vec::new(),
            selection: Vec::new(),
            select_all: true,

            drag_start: [-100.0, -100.0],
            drag_end: [-100.0, -100.0],
        }
    }

    fn add_instance(&mut self, instance: Instance) {
        self.instances.push(instance);
        self.selection.push(self.select_all);
    }

    // if selecton < 0, add a new instance
    // else update the selected one
    pub fn pointed(&mut self, point: [f32; 2]) {
        let label = 1.0f32; // TODO: 0.0 for background

        if self.operation_mode == OptMode::NewInstance {
            // add a new instance
            self.add_instance(Instance::new_point(point[0], point[1], label));
        } else {
            // do not add new instance
            if self.selection_mode {
                // select the nearest instance
                self.find_instance(point);
            } else {
                // add prompt to selected isntance
                if self.operation_mode == OptMode::AddOn {
                    let selected = self.check_selection();
                    if selected >= 0 {
                        self.instances[selected as usize]
                            .add_point_label(point[0], point[1], label);
                    }
                }
            }
        }
    }

    // if selecton < 0, add a new instance
    // else update the selected one
    pub fn boxed(&mut self, bbox: [f32; 4], is_manual: bool) {
        let selection = if is_manual {
            self.check_selection()
        } else {
            -1
        };

        if selection < 0 {
            self.add_instance(Instance::new_box(bbox, is_manual));
        } else {
            self.instances[selection as usize].add_box(bbox, is_manual);
        }
    }

    pub fn add_yolo_boxes(&mut self, boxes: Vec<[f32; 4]>) {
        for bbox in boxes {
            self.boxed(bbox, false);
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

    // return -1 if no specific instance is selected
    // else return the index of the selected instance
    pub fn check_selection(&self) -> i32 {
        // go through the selection twice
        // Considering that the Vec is not big at all, this is fine.

        if self.select_all && self.selection.len() != 1 {
            return -1;
        }

        // find the number of true
        let count = self.selection.iter().filter(|&&x| x).count();

        if count > 1 || count == 0 {
            // if more than one or none is selected
            -1
        } else {
            // if only one is selected, find the index
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

    // use the input to find the nearest instance based on Instance::pos
    // set the selection for the instance to true
    // this would lead to show the only one instance found
    pub fn find_instance(&mut self, pos: [f32; 2]) {
        let idx = self
            .instances
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let a_dist = a.get_distance(pos);
                let b_dist = b.get_distance(pos);
                a_dist.partial_cmp(&b_dist).unwrap()
            })
            .unwrap()
            .0;

        // set all selection to false
        self.selection = vec![false; self.selection.len()];
        self.selection[idx] = true;
        self.select_all = false;
    }

    pub fn remove_instance(&mut self, idx: usize) {
        self.instances.remove(idx);
        self.selection.remove(idx);
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

impl fmt::Display for OptMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptMode::None => write!(f, "None"),
            OptMode::NewInstance => write!(f, "New instance"),
            OptMode::AddOn => write!(f, "Add on"),
            OptMode::Delete => write!(f, "Delete"),
        }
    }
}
