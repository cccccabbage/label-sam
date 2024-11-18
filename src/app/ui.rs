mod instance;
mod state;

use super::threads::{Command, Return};
use imageproc::drawing::Canvas;
pub use instance::Outline;
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

        // handle return values
        if let Ok(ret) = self.receiver.try_recv() {
            match ret {
                Return::Img(img) => {
                    let crate::app::threads::image_loader::Image {
                        data,
                        path,
                        size,
                        file_size,
                    } = img;
                    self.state.img = Some(data);
                    self.state.img_ori_size = Some(size);
                    self.state.img_path = Some(path);
                    self.state.img_file_size = Some(file_size);

                    self.running = false;
                }
                Return::Mask(ins_masks) => {
                    self.running = false;

                    for (i, mask) in ins_masks.into_iter().enumerate() {
                        self.state.instances[i].add_mask(mask);
                    }
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
            receiver,

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
    fn draw_info_column(&mut self, ctx: &egui::Context) {
        SidePanel::right("infos").show(ctx, |ui| {
            self.draw_img_info(ui);

            ui.separator();

            self.draw_instance_info(ui);

            // TODO
            // Prompt Section
            // TODO
            // File Section
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

                ui.separator();
                ui.checkbox(&mut self.state.selection_mode, "Selection Mode");
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
                            if response.clicked() {
                                let p = self.normalize(mouse_pos.into());
                                self.img_pointed(p);
                            }
                        }
                        PromptType::Box => {
                            if response.drag_started() {
                                self.state.drag_start = mouse_pos.into();
                                self.state.drag_end = mouse_pos.into();
                            } else if response.drag_stopped() {
                                self.state.drag_end = mouse_pos.into();

                                let bbox = Rect::from_two_pos(
                                    self.normalize(self.state.drag_start).into(),
                                    self.normalize(self.state.drag_end).into(),
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
            self.draw_outline(ui.painter());
        });
    }

    fn draw_prompts(&self, painter: &Painter) {
        painter.rect(
            egui::Rect::from_two_pos(self.state.drag_start.into(), self.state.drag_end.into()),
            1.0,
            egui::Color32::TRANSPARENT,
            egui::Stroke::new(2.0, egui::Color32::RED),
        );

        self.state.draw_prompts(painter);
    }

    fn draw_outline(&self, painter: &Painter) {
        self.state.draw_outline(painter);
    }

    fn draw_instance_info(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            if ui
                .checkbox(&mut self.state.select_all, "Select All")
                .changed()
            {
                self.state.change_select_all();
            }

            for (i, b) in self.state.selection.iter_mut().enumerate() {
                ui.checkbox(b, format!("Instance {}", i));
                if !(*b) {
                    self.state.select_all = false;
                }
            }
        });
    }

    fn draw_img_info(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Image Info Section
            ui.label("Image Info");
            ui.vertical(|ui| {
                match &self.state.img {
                    Some(_) => (),
                    None => {
                        // noting yet
                        ui.label(format!("No Image loaded"));
                    }
                }
                match &self.state.img_path {
                    Some(path) => {
                        ui.label(format!("Path: {}", path.to_str().unwrap()));
                    }
                    None => (),
                }
                match &self.state.img_ori_size {
                    Some(size) => {
                        ui.label(format!("Image Size: {} {}", size[0], size[1]));
                    }
                    None => (),
                }
                match &self.state.img_file_size {
                    Some(size) => {
                        ui.label(format!("File Size: {:.2} KB", size / 1024.0));
                    }
                    None => (),
                }
            });
        });
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
        let instances_prompts = self.state.format_prompts();
        self.sender
            .send(Command::Segment(instances_prompts))
            .expect("Failed to send command Segment");
    }

    fn detect(&mut self) {
        if self.running {
            println!("task running, try again later");
            return;
        } else {
            self.running = true;
        }

        self.sender
            .send(Command::Detect)
            .expect("Failed to send command Detect");
    }

    fn img_pointed(&mut self, point: [f32; 2]) {
        self.state.add_point_label(point, 1.0); // TOOD: label 0.0 for background
    }

    fn img_boxed(&mut self, bbox: [f32; 4]) {
        self.state.add_box(bbox, true);
    }
}

// private, utils
impl UiData {
    fn normalize(&self, point: [f32; 2]) -> [f32; 2] {
        let size = self.state.img.as_ref().unwrap().dimensions();
        let [isx, isy] = [size.0 as f32, size.1 as f32];
        let [ipx, ipy] = self.state.img_pos.unwrap();
        [(point[0] - ipx) / isx, (point[1] - ipy) / isy]
    }
}
