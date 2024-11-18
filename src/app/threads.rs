pub mod image_loader;

use super::model::sam::prompt::Prompt;
use super::ui::Outline;

use std::{
    fmt,
    sync::mpsc::{Receiver, Sender},
    thread,
};

#[derive(Debug)]
pub enum Command {
    ReadImage,
    Segment(Vec<Vec<Prompt>>),
    Detect,
}

pub enum Return {
    Img(image_loader::Image),
    Mask(Vec<Outline>),
    BBox(Vec<[f32; 4]>),

    Void,
}

pub struct ComputationData {
    img: Option<image_loader::Image>,
    model: super::model::Models,

    sender: Sender<Return>,
    receiver: Receiver<Command>,
}

// public
impl ComputationData {
    pub fn new(sender: Sender<Return>, receiver: Receiver<Command>) -> Self {
        ComputationData {
            sender,
            receiver,

            model: super::model::Models::new(),
            img: None,
        }
    }
}

pub fn run(mut data: ComputationData) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Kill task
    thread::spawn(move || {
        while let Ok(task) = data.receiver.recv() {
            let msg = task.to_string();
            data.run_task(task)
                .expect(format!("Failed to run task: {msg}").as_str());
        }
    });
    Ok(())
}

// private
impl ComputationData {
    fn run_task(&mut self, task: Command) -> Result<(), Box<dyn std::error::Error>> {
        let timer = std::time::Instant::now();
        let msg = task.to_string();
        let ret = match task {
            Command::ReadImage => self.read_image(),
            Command::Segment(s) => self.segment(s),
            Command::Detect => self.detect(),
        };
        Self::time(timer, &msg);
        self.sender.send(ret).expect("Failed to send Return");
        Ok(())
    }

    fn read_image(&mut self) -> Return {
        let img = image_loader::Image::load("tests/imgs/0000.jpg".into()).unwrap();

        self.img = Some(img);
        self.model.embed(&self.img.as_ref().unwrap().data);

        Return::Img(self.img.clone().unwrap()) // TODO: clone here
    }

    fn segment(&mut self, instances_prompts: Vec<Vec<Prompt>>) -> Return {
        match &self.img {
            Some(img) => {
                self.model.embed(&img.data); // if embeded, this will do nothing

                let mut outlines = Vec::new();

                for prompts in instances_prompts {
                    let mask = self.model.generate_mask(prompts).to_luma8();
                    outlines.push(Outline::from(&mask).normalize(img.size));
                }

                Return::Mask(outlines)
            }
            None => {
                println!("No image to segment");
                Return::Void
            }
        }
    }

    fn detect(&mut self) -> Return {
        let img_ref = self.img.as_ref();
        let r = match img_ref {
            Some(img_ref) => {
                // the points for boxes have already been normalized
                let boxes = self.model.detect(&img_ref.data);

                let mut prompts = Vec::new();
                for (bb, _) in boxes.iter() {
                    prompts.push(bb.into());
                }
                Return::BBox(prompts)
            }
            None => Return::Void,
        };

        r
    }
}

// private, utils
impl ComputationData {
    fn time(timer: std::time::Instant, msg: &str) {
        println!("Time elapsed for {msg}: {:?}", timer.elapsed());
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::ReadImage => write!(f, "Read Image"),
            Command::Detect => write!(f, "Detect"),
            Command::Segment(_) => write!(f, "Segment"),
        }
    }
}
