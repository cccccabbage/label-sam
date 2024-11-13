mod state;

use super::threads::{Command, Return};
use state::{PromptHover, PromptType, UiState};

use egui::{
    CentralPanel, ColorImage, Painter, Rect, Sense, SidePanel, TextureOptions, TopBottomPanel,
};
use strum::IntoEnumIterator;

use std::sync::mpsc::{Receiver, Sender};

pub struct UiData {
    sender: Sender<Command>,
    receiver: Receiver<Return>,

    state: UiState,
}

impl eframe::App for UiData {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw_info_column(ctx);

        self.draw_button_row(ctx);

        self.draw_img_area(ctx);

        if let Ok(img) = self.receiver.try_recv() {
            match img {
                Return::Img(img) => {
                    self.state.img = Some(img);
                }
                Return::Void => (),
            }
        }

        ctx.request_repaint();
    }
}

// private
impl UiData {
    pub fn new(sender: Sender<Command>, receiver: Receiver<Return>) -> Self {
        UiData {
            sender,
            receiver, // change to

            state: UiState::new(),
        }
    }

    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        eframe::run_native(
            "app_name",
            eframe::NativeOptions {
                viewport: egui::ViewportBuilder {
                    position: None,
                    inner_size: Some(egui::vec2(2560.0, 1440.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            Box::new(move |_cc| Ok(Box::new(self))),
        )
        .unwrap();
        Ok(())
    }
}

// private
impl UiData {
    fn draw_info_column(&self, ctx: &egui::Context) {
        SidePanel::right("infos").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("Image Info");
                match &self.state.img {
                    Some(img) => {
                        // there is an image
                        ui.label(format!("Width: {}", img.width()));
                        ui.label(format!("Height: {}", img.height()));
                    }
                    None => {
                        // noting yet
                        ui.label(format!("No Image loaded"));
                    }
                }
            });
        });
    }

    fn draw_button_row(&mut self, ctx: &egui::Context) {
        // the whole button row
        TopBottomPanel::top("Button Area").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // basic functions
                if ui.button("Read Image").clicked() {
                    self.sender
                        .send(Command::ReadImage)
                        .expect("Failed to send command ReadImage");
                }
                if ui.button("Segment").clicked() {
                    self.segment();
                }
                if ui.button("Detect").clicked() {
                    self.detect();
                }

                ui.separator();

                // selection for prompt type
                ui.label("Prompt: ");
                for variant in PromptType::iter() {
                    ui.radio_value(&mut self.state.prompt_type, variant, variant.to_string());
                }

                ui.separator();

                // selection for prompt hover
                ui.label("Show Prompt: ");
                for variant in PromptHover::iter() {
                    ui.radio_value(&mut self.state.prompt_hover, variant, variant.to_string());
                }
            });
        });
    }

    fn draw_img_area(&mut self, ctx: &egui::Context) {
        // acquire the mouse position
        let mouse_pos = ctx
            .input(|i| i.pointer.hover_pos())
            .unwrap_or([0.0, 0.0].into())
            .to_vec2();

        // image area
        CentralPanel::default().show(ctx, |ui| {
            match &self.state.img {
                Some(img) => {
                    // load img
                    let size = [img.width() as usize, img.height() as usize];
                    let img_data = img.to_rgb8().into_raw();
                    let img = ColorImage::from_rgb(size, &img_data);
                    let texture = ctx.load_texture("image", img, TextureOptions::default());

                    // show img, responds to click and drag
                    let response = ui.image(&texture).interact(Sense::click_and_drag());

                    // handle input
                    match self.state.prompt_type {
                        PromptType::None => (),
                        PromptType::Point => {
                            let size_p = egui::vec2(size[0] as f32, size[1] as f32);
                            if response.clicked() {
                                // get the position of the click in the image, normailzed
                                let click_pos = (mouse_pos - response.rect.min.to_vec2()) / size_p;
                                self.img_pointed(mouse_pos, click_pos);
                            }
                        }
                        PromptType::Box => {
                            if response.drag_started() {
                                self.state.drag_start = mouse_pos.into();
                                self.state.drag_end = mouse_pos.into();
                            } else if response.drag_stopped() {
                                self.state.drag_end = mouse_pos.into();

                                let bbox = Rect::from_two_pos(
                                    self.state.drag_start.into(),
                                    self.state.drag_end.into(),
                                );
                                self.img_boxed(
                                    [
                                        bbox.min.x / size[0] as f32,
                                        bbox.min.y / size[1] as f32,
                                        bbox.max.x / size[0] as f32,
                                        bbox.max.y / size[1] as f32,
                                    ],
                                    [bbox.min.x, bbox.min.y, bbox.max.x, bbox.max.y],
                                );
                            } else if response.dragged() {
                                self.state.drag_end = mouse_pos.into();
                            }
                        }
                    }
                }
                // No image yet or waiting for feedback like segment
                None => {
                    ui.label(&self.state.img_label);
                }
            }

            // draw hovers
            let painter = ui.painter();
            match self.state.prompt_hover {
                PromptHover::None => (),
                PromptHover::Point => self.draw_point_hovers(painter),
                PromptHover::Box => self.draw_box_hovers(painter),
                PromptHover::All => {
                    self.draw_point_hovers(painter);
                    self.draw_box_hovers(painter);
                }
            }
        });
    }

    fn draw_point_hovers(&self, painter: &Painter) {
        for p in self.state.prompt_points.iter() {
            painter.circle(
                p.into(),
                3.0,
                egui::Color32::RED,
                egui::Stroke::new(1.0, egui::Color32::BLACK),
            );
        }
    }

    fn draw_box_hovers(&self, painter: &Painter) {
        for bb in self.state.prompt_boxes.iter() {
            let p1 = [bb[0], bb[1]];
            let p2 = [bb[2], bb[3]];
            painter.rect(
                egui::Rect::from_two_pos(p1.into(), p2.into()),
                1.0,
                egui::Color32::from_white_alpha(0),
                egui::Stroke::new(2.0, egui::Color32::RED),
            );
        }
        painter.rect(
            egui::Rect::from_two_pos(self.state.drag_start.into(), self.state.drag_end.into()),
            1.0,
            egui::Color32::from_white_alpha(0),
            egui::Stroke::new(2.0, egui::Color32::RED),
        );
    }
}

// private, backend thread related
impl UiData {
    fn segment(&mut self) {
        self.sender
            .send(Command::Segment)
            .expect("Failed to send command Segment");

        self.state.img = None;
        self.state.img_label = "Segmenting...".to_string();
    }

    fn detect(&mut self) {
        self.sender
            .send(Command::Detect)
            .expect("Failed to send command Detect");

        self.state.img = None;
        self.state.img_label = "Detecting...".to_string();
    }

    fn add_point(&mut self, pos: egui::Vec2) {
        self.sender
            .send(Command::AddPoint(pos.into()))
            .expect("Failed to send command AddPoint");
    }

    fn img_pointed(&mut self, mouse_pos: egui::Vec2, click_pos: egui::Vec2) {
        self.add_point(click_pos);
        self.state.add_point(mouse_pos);
    }

    fn img_boxed(&mut self, prompt_bbox: [f32; 4], hover_bbox: [f32; 4]) {
        self.sender
            .send(Command::AddBox(prompt_bbox))
            .expect("Failed to send command AddBox");

        self.state.add_box(hover_bbox);
    }
}
