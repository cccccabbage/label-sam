use sam::prompt::Prompt;

pub mod sam;
pub mod yolo;

pub struct Models {
    sam: sam::SAMmodel,
    yolo: yolo::YOLOmodel,

    embeded: bool,
}

impl Models {
    pub fn new() -> Self {
        Self {
            sam: sam::SAMmodel::new(),
            yolo: yolo::YOLOmodel::new(),

            embeded: false,
        }
    }

    // The values in bounding boxes have already been normalized
    pub fn detect(&self, img: &image::DynamicImage) -> Vec<(yolo::BoundingBox, f32)> {
        self.yolo.forward(img)
    }

    pub fn embed(&mut self, img: &image::DynamicImage) {
        if !self.embeded {
            self.sam.embed(img).unwrap();
            self.embeded = true;
        }
    }

    // the prompt should be normalized
    pub fn generate_mask(&self, prompts: Vec<Prompt>) -> image::DynamicImage {
        assert!(self.embeded);
        self.sam.generate_mask(prompts)
    }
}
