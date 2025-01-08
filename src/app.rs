mod model;
mod threads;
mod ui;

use crate::config::Config;
use std::sync::mpsc;
use threads::{Command, Return};

pub struct App {
    config: Config,
}

impl App {
    pub fn new(config: Config) -> Self {
        App { config }
    }

    pub fn run(&self) {
        let (task_sender, task_reciver) = mpsc::channel::<Command>();
        let (result_sender, result_reciver) = mpsc::channel::<Return>();

        threads::run(threads::ComputationData::new(
            result_sender,
            task_reciver,
            &self.config.yolo_path,
            &self.config.sam_e_path,
            &self.config.sam_d_path,
        ))
        .expect("Create thread failed");

        // TODO: a copy here
        ui::UiData::new(task_sender.clone(), result_reciver)
            .run()
            .expect("Run Ui Error");

        task_sender
            .send(threads::Command::End)
            .expect("End Backend Thread Error");
    }
}

#[allow(unused)]
pub fn test_sam() -> Result<(), Box<dyn std::error::Error>> {
    use model::sam::prompt::Prompt;

    let mut sam = model::sam::SAMmodel::new(); // load model

    let img = image::open("tests/imgs/0000.jpg").unwrap();

    let prompt = Prompt::new_point(0.36, 0.39f32, 1.0f32);

    let i = sam.forward(&img, prompt); // run forward
    i.save("sam-result-test.png").unwrap();

    Ok(())
}

#[allow(unused)]
pub fn test_yolo() -> Result<(), Box<dyn std::error::Error>> {
    let yolo = model::yolo::YOLOmodel::new(); // load model

    let img = image::open("tests/imgs/0000.jpg").unwrap();

    let boxes = yolo.forward(&img); // run forward

    let mut last_conf = 1.0f32;
    for (_bbox, conf) in &boxes {
        assert!(*conf < last_conf);
        last_conf = *conf;
    }

    Ok(())
}
