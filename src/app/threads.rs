use super::model::sam::Prompt;
use super::model::yolo::BoundingBox;

use image::DynamicImage;
use std::{
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
    thread,
};

#[derive(Debug)]
pub enum Command {
    ReadImage,
    Segment,
    Detect,

    AddPoint([f32; 2]),
}

pub enum Return {
    Img(Arc<DynamicImage>),

    Void,
}

pub struct ComputationData {
    img: Option<Arc<DynamicImage>>,
    mask: Option<Arc<DynamicImage>>,
    prompts: Option<Vec<Prompt>>,
    detected: bool,

    model: super::model::Models,

    sender: Sender<Return>,
    receiver: Receiver<Command>,
}

impl ComputationData {
    pub fn new(sender: Sender<Return>, receiver: Receiver<Command>) -> Self {
        ComputationData {
            sender,
            receiver,

            model: super::model::Models::new(),
            img: None,
            mask: None,
            prompts: None,
            detected: false,
        }
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Kill task
        thread::spawn(move || {
            while let Ok(task) = self.receiver.recv() {
                self.run_task(&task)
                    .expect(format!("Failed to run task: {:?}", task).as_str());
            }
        });
        Ok(())
    }

    fn run_task(&mut self, task: &Command) -> Result<(), Box<dyn std::error::Error>> {
        let timer = std::time::Instant::now();
        let msg = match task {
            Command::ReadImage => "Read Image",
            Command::Segment => "Segment",
            Command::Detect => "Detect",
            Command::AddPoint(_) => "Add Point",
        };
        let ret = match task {
            Command::ReadImage => self.read_image(),
            Command::Segment => self.segment(),
            Command::Detect => self.detect(),
            Command::AddPoint(p) => self.add_point(*p),
        };
        Self::time(timer, msg);
        self.sender.send(ret).expect("Failed to send Return");
        Ok(())
    }

    fn read_image(&mut self) -> Return {
        let img = Arc::new(image::open("tests/imgs/0000.jpg").unwrap());

        self.img = Some(img.clone());
        self.model.embed(img.as_ref());

        Return::Img(img)
    }

    fn segment(&mut self) -> Return {
        let img_ref = self.img.as_ref();
        match img_ref {
            Some(img_ref) => {
                self.model.embed(img_ref);
                let mut masks = Vec::new();
                for p in self.prompts.as_ref().unwrap() {
                    masks.push(self.model.generate_mask(p).to_luma8());
                }
                let mask = crate::utils::mask_or(masks);
                let mask = Arc::new(DynamicImage::from(mask));

                self.mask = Some(mask.clone());
                Return::Img(mask.clone())
            }
            None => Return::Void,
        }
    }

    fn detect(&mut self) -> Return {
        if self.detected {
            Return::Void
        } else {
            let img_ref = self.img.as_ref();
            let r = match img_ref {
                Some(img_ref) => {
                    // the points for boxes have already been normalized
                    let boxes = self.model.detect(img_ref);
                    let boxes = Arc::new(boxes);

                    // take the first box
                    let (bb, _) = boxes.first().unwrap();
                    let mut prompt = Prompt::new();
                    let BoundingBox { x1, y1, x2, y2 } = bb;
                    prompt.add_box(*x1, *y1, *x2, *y2);
                    match self.prompts.as_mut() {
                        Some(prompts) => {
                            prompts.push(prompt);
                        }
                        None => {
                            self.prompts = Some(vec![prompt]);
                        }
                    };

                    // deal with the rest boxes
                    for (bb, _) in boxes.iter().skip(1) {
                        let mut prompt = Prompt::new();
                        let BoundingBox { x1, y1, x2, y2 } = bb;
                        prompt.add_box(*x1, *y1, *x2, *y2);
                        let prompts = self.prompts.as_mut().unwrap();
                        prompts.push(prompt);
                    }

                    Return::Img(img_ref.clone())
                }
                None => Return::Void,
            };

            r
        }
    }

    fn add_point(&mut self, point: [f32; 2]) -> Return {
        let mut prompt = Prompt::new();
        prompt.add_point(point[0], point[1], 1.0);
        match self.prompts.as_mut() {
            Some(prompts) => {
                prompts.push(prompt);
            }
            None => {
                self.prompts = Some(vec![prompt]);
            }
        };

        Return::Void
    }

    fn time(timer: std::time::Instant, msg: &str) {
        println!("Time elapsed for {msg}: {:?}", timer.elapsed());
    }
}
