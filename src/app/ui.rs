mod state;

use super::threads::{Command, Return};

use state::UiState;

use egui::{CentralPanel, ColorImage, Sense, SidePanel, TextureOptions, TopBottomPanel};
// use image::DynamicImage;

use std::sync::{
    mpsc::{Receiver, Sender},
    // Arc,
};

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
                    self.sender
                        .send(Command::Segment)
                        .expect("Failed to send command Segment");

                    self.state.img = None;
                    self.state.img_label = "Segmenting...".to_string();
                }

                if ui.button("Detect").clicked() {
                    self.sender
                        .send(Command::Detect)
                        .expect("Failed to send command Detect");

                    self.state.img = None;
                    self.state.img_label = "Detecting...".to_string();
                }

                ui.separator();

                for (n, b) in self
                    .state
                    .checkbox_names
                    .iter()
                    .zip(self.state.checkbox_states.iter_mut())
                {
                    ui.checkbox(b, *n);
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
                    let pos_in_img = mouse_pos - response.rect.min;
                    let normalized_pos = pos_in_img / size_p;

                    if self.state.checkbox_states[0] {
                        self.sender
                            .send(Command::AddPoint(normalized_pos.into()))
                            .expect("Failed to send command AddPoint");
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
}
