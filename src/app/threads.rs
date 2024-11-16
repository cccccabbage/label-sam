use image::DynamicImage;

use std::{
    fmt,
    sync::mpsc::{Receiver, Sender},
    thread,
};

#[derive(Debug)]
pub struct SegmentData {
    pub points: Vec<[f32; 2]>,
    pub labels: Vec<f32>,
    pub boxes: Vec<[f32; 4]>,
}

#[derive(Debug)]
pub struct DetectData {
    pub img_pos: [f32; 2],
    pub img_size: [f32; 2],
}

#[derive(Debug)]
pub enum Command {
    ReadImage,
    Segment(SegmentData),
    Detect(DetectData),
}

pub enum Return {
    Img(DynamicImage),
    BBox(Vec<[f32; 4]>),

    Void,
}

pub struct ComputationData {
    img: Option<DynamicImage>,
    mask: Option<DynamicImage>,

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
            mask: None,
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
            Command::Detect(d) => self.detect(d),
        };
        Self::time(timer, &msg);
        self.sender.send(ret).expect("Failed to send Return");
        Ok(())
    }

    fn read_image(&mut self) -> Return {
        let img = image::open("tests/imgs/0000.jpg").unwrap();

        self.img = Some(img);
        self.model.embed(self.img.as_ref().unwrap());

        Return::Img(self.img.clone().unwrap()) // TODO: clone here
    }

    fn segment(&mut self, data: SegmentData) -> Return {
        // let img_ref = self.img.as_ref();
        match &self.img {
            Some(img) => {
                self.model.embed(img); // if embeded, this will do nothing

                let mut masks = Vec::new();
                let SegmentData {
                    points,
                    labels,
                    boxes,
                } = data;

                for p_and_l in points.iter().zip(labels.iter()) {
                    masks.push(self.model.generate_mask(p_and_l.into()).to_luma8());
                }
                for bbox in boxes {
                    masks.push(self.model.generate_mask(bbox.into()).to_luma8());
                }

                let mask = crate::utils::mask_or(masks);
                let mask = DynamicImage::from(mask);

                self.mask = Some(mask);

                // TODO: return different masks for each instance
                Return::Img(self.mask.clone().unwrap())
            }
            None => {
                println!("No image to segment");
                Return::Void
            }
        }
    }

    fn detect(&mut self, data: DetectData) -> Return {
        let img_ref = self.img.as_ref();
        let r = match img_ref {
            Some(img_ref) => {
                // the points for boxes have already been normalized
                let boxes = self.model.detect(img_ref);

                let DetectData { img_pos, img_size } = data;

                let mut prompts = Vec::new();
                for (bb, _) in boxes.iter() {
                    let bb: [f32; 4] = bb.into();
                    let bb = [
                        bb[0] * img_size[0] + img_pos[0],
                        bb[1] * img_size[1] + img_pos[1],
                        bb[2] * img_size[0] + img_pos[0],
                        bb[3] * img_size[1] + img_pos[1],
                    ];
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
            Command::Detect(_) => write!(f, "Detect"),
            Command::Segment(_) => write!(f, "Segment"),
        }
    }
}
