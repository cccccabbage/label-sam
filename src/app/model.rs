use sam::prompt::Prompt;

pub mod sam;
pub mod yolo;

pub struct Models {
    sam: sam::SAMmodel,
    yolo: yolo::YOLOmodel,

    embeded: bool,
}

impl Models {
    pub fn new(yolo_path: &str, sam_e_path: &str, sam_d_path: &str) -> Self {
        Self {
            sam: sam::SAMmodel::new_path(sam_e_path, sam_d_path),
            yolo: yolo::YOLOmodel::new_path(yolo_path),

            embeded: false,
        }
    }

    // The values in bounding boxes have already been normalized
    pub fn detect(&self, img: &image::DynamicImage) -> Vec<(yolo::BoundingBox, f32)> {
        self.yolo.forward(img)
    }

    pub fn embed(&mut self, img: &image::DynamicImage) {
        self.sam.embed(img).unwrap();
        self.embeded = true;
    }

    // the prompt should be normalized
    pub fn generate_mask(&self, prompts: Vec<Prompt>) -> image::DynamicImage {
        assert!(self.embeded);
        self.sam.generate_mask(prompts)
    }
}
