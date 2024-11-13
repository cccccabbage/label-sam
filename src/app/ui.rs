mod state;

use super::threads::{Command, Return};
use state::{PromptType, UiState};

use egui::{CentralPanel, ColorImage, Sense, SidePanel, TextureOptions, TopBottomPanel};
use strum::IntoEnumIterator;

use std::sync::mpsc::{Receiver, Sender};

pub struct UiData {
    sender: Sender<Command>,
    receiver: Receiver<Return>,

    state: UiState,
}

impl eframe::App for UiData {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mouse_pos = ctx
            .input(|i| i.pointer.hover_pos())
            .unwrap_or([0.0, 0.0].into());

        // info column
        SidePanel::right("infos").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("Image Info");
                match &self.state.img {
                    Some(img) => {
                        ui.label(format!("Width: {}", img.width()));
                        ui.label(format!("Height: {}", img.height()));
                    }
                    None => {
                        ui.label(format!("No Image loaded"));
                    }
                }
            });
        });

        TopBottomPanel::top("Button Area").show(ctx, |ui| {
            ui.horizontal(|ui| {
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

                for variant in PromptType::iter() {
                    ui.radio_value(&mut self.state.prompt_type, variant, variant.to_string());
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| match &self.state.img {
            Some(img) => {
                // load img
                let size = [img.width() as usize, img.height() as usize];
                let img_data = img.to_rgb8().into_raw();
                let img = ColorImage::from_rgb(size, &img_data);
                let texture = ctx.load_texture("image", img, TextureOptions::default());

                // show img
                let response = ui.image(&texture).interact(Sense::click_and_drag());

                // handle input
                let size_p = egui::vec2(size[0] as f32, size[1] as f32);
                if response.clicked() {
                    let mouse_pos = (mouse_pos - response.rect.min) / size_p;
                    match self.state.prompt_type {
                        PromptType::Point => {
                            self.add_point(mouse_pos);
                        }
                        PromptType::Box => {
                            println!("It's in Box Prompt Mode, click will do nothing. You may need to drag for a box.");
                        }
                        PromptType::Void =>(),
                    }
                } else if response.dragged() {
                    println!("22");
                }
            }
            None => {
                ui.label(&self.state.img_label);
            }
        });

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

impl UiData {
    pub fn new(sender: Sender<Command>, receiver: Receiver<Return>) -> Self {
        UiData {
            sender,
            receiver,

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
}
