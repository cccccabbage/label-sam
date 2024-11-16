mod state;

use super::threads::{Command, DetectData, Return, SegmentData};
use imageproc::drawing::Canvas;
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
    running: bool, // when a task is running, disable the buttons
}

impl eframe::App for UiData {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw_info_column(ctx);

        self.draw_button_row(ctx);

        self.draw_img_area(ctx);

        if let Ok(ret) = self.receiver.try_recv() {
            match ret {
                Return::Img(img) => {
                    self.state.img = Some(img);
                    self.running = false;
                }
                Return::BBox(boxes) => {
                    self.state.add_yolo_boxes(boxes);
                    self.running = false;
                }
                Return::Void => self.running = false,
            }
        }

        ctx.request_repaint(); // TODO: this is not efficient
    }
}

// private
impl UiData {
    pub fn new(sender: Sender<Command>, receiver: Receiver<Return>) -> Self {
        UiData {
            sender,
            receiver, // change to

            state: UiState::new(),
            running: false,
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

// private, drawing related
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
                    self.read_img();
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
                    self.state.img_pos = Some(response.rect.min.into());

                    // handle input
                    match self.state.prompt_type {
                        PromptType::None => (),
                        PromptType::Point => {
                            // let size_p = egui::vec2(size[0] as f32, size[1] as f32);
                            if response.clicked() {
                                // get the position of the click in the image, normailzed
                                // let click_pos = (mouse_pos - response.rect.min.to_vec2()) / size_p;
                                self.img_pointed(mouse_pos.into());
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
                                self.img_boxed([bbox.min.x, bbox.min.y, bbox.max.x, bbox.max.y]);

                                self.state.drag_start = [-100.0, -100.0].into();
                                self.state.drag_end = [-100.0, -100.0].into();
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

            self.draw_prompts(ui.painter());
        });
    }

    fn draw_prompts(&self, painter: &Painter) {
        painter.rect(
            egui::Rect::from_two_pos(self.state.drag_start.into(), self.state.drag_end.into()),
            1.0,
            egui::Color32::TRANSPARENT,
            egui::Stroke::new(2.0, egui::Color32::RED),
        );

        match self.state.prompt_hover {
            PromptHover::Point | PromptHover::All => {
                for [x, y] in &self.state.points {
                    painter.circle(
                        egui::Pos2::new(*x, *y),
                        3.0,
                        egui::Color32::RED,
                        egui::Stroke::new(1.0, egui::Color32::BLACK),
                    );
                }
            }
            _ => (),
        }

        match self.state.prompt_hover {
            PromptHover::Box | PromptHover::All => {
                for [x1, y1, x2, y2] in &self.state.yolo_boxes {
                    painter.rect(
                        egui::Rect::from_two_pos(
                            egui::Pos2::new(*x1, *y1),
                            egui::Pos2::new(*x2, *y2),
                        ),
                        1.0,
                        egui::Color32::TRANSPARENT,
                        egui::Stroke::new(2.0, egui::Color32::RED),
                    );
                }

                for [x1, y1, x2, y2] in &self.state.boxes {
                    painter.rect(
                        egui::Rect::from_two_pos(
                            egui::Pos2::new(*x1, *y1),
                            egui::Pos2::new(*x2, *y2),
                        ),
                        1.0,
                        egui::Color32::TRANSPARENT,
                        egui::Stroke::new(2.0, egui::Color32::RED),
                    );
                }
            }
            _ => (),
        }
    }
}

// private, backend thread related
impl UiData {
    fn read_img(&mut self) {
        if self.running {
            println!("task running, try again later");
            return;
        } else {
            self.running = true;
        }

        self.sender
            .send(Command::ReadImage)
            .expect("Failed to send command ReadImage");
    }

    fn segment(&mut self) {
        if self.running {
            println!("task running, try again later");
            return;
        } else {
            self.running = true;
        }
        let size = self.state.img.as_ref().unwrap().dimensions();
        let [isx, isy] = [size.0 as f32, size.1 as f32];
        let [ipx, ipy] = self.state.img_pos.unwrap();

        let points = self.state.points.clone();
        let points = points
            .iter()
            .map(|[x, y]| [(x - ipx) / isx, (y - ipy) / isy])
            .collect();
        let labels = self.state.point_labels.clone();

        let size = self.state.img.as_ref().unwrap().dimensions();
        let size = [size.0 as f32, size.1 as f32];
        let pos = self.state.img_pos.unwrap();

        let boxes: Vec<[f32; 4]> = self
            .state
            .boxes
            .clone()
            .into_iter()
            .chain(self.state.yolo_boxes.clone().into_iter())
            .collect(); // the two vec of boxes are concatenated
        let boxes = boxes
            .iter()
            .map(|[x1, y1, x2, y2]| {
                [
                    (x1 - pos[0]) / size[0],
                    (y1 - pos[1]) / size[1],
                    (x2 - pos[0]) / size[0],
                    (y2 - pos[1]) / size[1],
                ]
            })
            .collect();

        let s = SegmentData {
            points,
            labels,
            boxes,
        };

        self.sender
            .send(Command::Segment(s))
            .expect("Failed to send command Segment");
    }

    fn detect(&mut self) {
        if self.running {
            println!("task running, try again later");
            return;
        } else {
            self.running = true;
        }

        let size = self.state.img.as_ref().unwrap().dimensions();
        let size = [size.0 as f32, size.1 as f32];
        self.sender
            .send(Command::Detect(DetectData {
                img_size: size,
                img_pos: self.state.img_pos.unwrap(),
            }))
            .expect("Failed to send command Detect");
    }

    fn img_pointed(&mut self, point: [f32; 2]) {
        self.state.add_point_label(point, 1.0); // TOOD: label 0.0 for background
    }

    fn img_boxed(&mut self, bbox: [f32; 4]) {
        self.state.add_box(bbox, true);
    }
}
