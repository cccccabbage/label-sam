use super::state::PromptHover;
use crate::app::model::sam::prompt::Prompt;

use image::GrayImage;

pub struct Instance {
    pub mask: Option<Outline>,
    pub prompts: Vec<Prompt>,

    pub box_manual: Vec<bool>,
}

#[derive(Clone)]
pub struct Outline(Vec<[f32; 2]>);

impl Outline {
    pub fn from(mask: &GrayImage) -> Self {
        let outline = crate::utils::extract_outline(mask);
        Outline(outline)
    }

    pub fn normalize(mut self, img_size: [f32; 2]) -> Self {
        for point in &mut self.0 {
            point[0] /= img_size[0];
            point[1] /= img_size[1];
        }

        self
    }
}

impl Instance {
    pub fn new() -> Self {
        Instance {
            mask: None,
            prompts: Vec::new(),
            box_manual: Vec::new(),
        }
    }

    pub fn add_point_label(&mut self, x: f32, y: f32, label: f32) {
        self.prompts.push(Prompt::new_point(x, y, label));
    }

    pub fn add_box(&mut self, bbox: [f32; 4], is_manual: bool) {
        self.prompts
            .push(Prompt::new_box(bbox[0], bbox[1], bbox[2], bbox[3]));
        self.box_manual.push(is_manual);
    }

    pub fn add_mask(&mut self, mask: Outline) {
        match &self.mask {
            Some(_) => {
                println!("mask already exists, overwriting");
            }
            None => {
                self.mask = Some(mask);
            }
        }
    }

    pub fn draw_prompt(
        &self,
        painter: &egui::Painter,
        hover: &PromptHover,
        img_size: &[f32; 2],
        img_pos: &[f32; 2],
    ) {
        let h = *hover;
        for (i, prompt) in self.prompts.iter().enumerate() {
            match prompt {
                Prompt::Point((p, l)) => {
                    if h == PromptHover::All || h == PromptHover::Point {
                        let c = if *l == 1.0 {
                            // 1.0 for front and 0.0 for back
                            egui::Color32::RED
                        } else {
                            egui::Color32::GREEN
                        };

                        let [x, y] = Self::denormalize(*p, *img_size, *img_pos);

                        painter.circle(
                            egui::Pos2::new(x, y),
                            3.0,
                            c,
                            egui::Stroke::new(1.0, egui::Color32::BLACK),
                        );
                    }
                }

                Prompt::Box([x1, y1, x2, y2]) => {
                    if h == PromptHover::All || h == PromptHover::Box {
                        let c = if self.box_manual[i] {
                            egui::Color32::RED
                        } else {
                            egui::Color32::GREEN
                        };

                        let p1 = [*x1, *y1];
                        let p2 = [*x2, *y2];
                        let p1 = Self::denormalize(p1, *img_size, *img_pos);
                        let p2 = Self::denormalize(p2, *img_size, *img_pos);

                        painter.rect(
                            egui::Rect::from_min_max(p1.into(), p2.into()),
                            1.0,
                            egui::Color32::TRANSPARENT,
                            egui::Stroke::new(1.0, c),
                        );
                    }
                }
            }
        }
    }

    pub fn draw_outline(&self, painter: &egui::Painter, img_size: &[f32; 2], img_pos: &[f32; 2]) {
        if let Some(mask) = &self.mask {
            for p in &mask.0 {
                let p = Self::denormalize(*p, *img_size, *img_pos);
                painter.circle_filled(p.into(), 1.0, egui::Color32::LIGHT_YELLOW);
            }
        }
    }
}

impl Instance {
    fn denormalize(p: [f32; 2], scale: [f32; 2], delta: [f32; 2]) -> [f32; 2] {
        [p[0] * scale[0] + delta[0], p[1] * scale[1] + delta[1]]
    }
}
