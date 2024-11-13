use image::{imageops::FilterType, GenericImageView};
use ndarray::{Array, Axis, Dim};
use ort::{inputs, CUDAExecutionProvider, GraphOptimizationLevel, Session, SessionOutputs};

// the trained yolo model has a input like this, so DO NOT change this.
const INPUT_H: u32 = 640;
const INPUT_W: u32 = 640;

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl BoundingBox {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        BoundingBox {
            x1: x1.min(x2),
            y1: y1.min(y2),
            x2: x1.max(x2),
            y2: y1.max(y2),
        }
    }

    pub fn normalize(self, w: f32, h: f32) -> Self {
        let Self { x1, y1, x2, y2 } = self;
        BoundingBox {
            x1: x1 / h,
            y1: y1 / w,
            x2: x2 / h,
            y2: y2 / w,
        }
    }

    // pub fn center(&self) -> (f32, f32) {
    //     ((self.x1 + self.x2) / 2.0, (self.y1 + self.y2) / 2.0)
    // }
}

#[derive(Debug)]
pub struct YOLOmodel {
    model: Session,
}

impl YOLOmodel {
    pub fn new() -> Self {
        Self::new_path("weights/yolov8s-trained.onnx")
    }

    pub fn new_path(p: &str) -> Self {
        let model = Session::builder()
            .unwrap()
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .unwrap()
            .with_execution_providers([CUDAExecutionProvider::default().build()])
            .unwrap()
            .with_intra_threads(4)
            .unwrap()
            .commit_from_file(p)
            .expect("Error load YOLO model");

        Self { model }
    }

    fn preprocess(img: &image::DynamicImage) -> Array<f32, Dim<[usize; 4]>> {
        let img = img.resize_exact(INPUT_W, INPUT_H, FilterType::CatmullRom);
        let mut input = Array::zeros((1, 3, INPUT_H as usize, INPUT_W as usize));
        for pixel in img.pixels() {
            let x = pixel.0 as _;
            let y = pixel.1 as _;
            let [r, g, b, _] = pixel.2 .0;

            input[[0, 0, y, x]] = (r as f32) / 255.0;
            input[[0, 1, y, x]] = (g as f32) / 255.0;
            input[[0, 2, y, x]] = (b as f32) / 255.0;
        }

        input
    }

    pub fn forward(&self, img: &image::DynamicImage) -> Vec<(BoundingBox, f32)> {
        let input = Self::preprocess(img);

        let outputs: SessionOutputs = self
            .model
            .run(inputs!["images" => input.view()].expect("Error in format YOLO inputs"))
            .expect("Error in YOLO format");

        Self::postprocess(&outputs).expect("Error in YOLO postprocess")
    }

    fn postprocess(
        outputs: &SessionOutputs,
    ) -> Result<Vec<(BoundingBox, f32)>, Box<dyn std::error::Error>> {
        let output = outputs["output0"]
            .try_extract_tensor::<f32>()?
            .t()
            .into_owned();

        let mut boxes = Vec::new();
        for row in output.axis_iter(Axis(0)) {
            let row: Vec<_> = row.iter().copied().collect();
            let (_class_id, prob) = row
                .iter()
                .skip(4)
                .enumerate()
                .map(|(index, value)| (index, *value))
                .reduce(|accum, row| if row.1 > accum.1 { row } else { accum })
                .unwrap();

            // TODO: flexible conf threshold
            if prob < 0.5 {
                continue;
            }

            let xc = row[0];
            let yc = row[1];
            let w = row[2];
            let h = row[3];
            boxes.push((
                BoundingBox::new(xc - w / 2.0, yc - h / 2.0, xc + w / 2.0, yc + h / 2.0)
                    .normalize(INPUT_W as f32, INPUT_H as f32),
                prob,
            ));
        }

        boxes.sort_by(|box1, box2| box2.1.total_cmp(&box1.1));
        let mut result = Vec::new();

        while !boxes.is_empty() {
            result.push(boxes[0]);
            boxes = boxes
                .iter()
                .filter(|box1| {
                    Self::intersection(&boxes[0].0, &box1.0) / Self::union(&boxes[0].0, &box1.0)
                        < 0.7 // TODO: flexible iou threshold
                })
                .copied()
                .collect();
        }

        Ok(result)
    }

    fn intersection(box1: &BoundingBox, box2: &BoundingBox) -> f32 {
        (box1.x2.min(box2.x2) - box1.x1.max(box2.x1))
            * (box1.y2.min(box2.y2) - box1.y1.max(box2.y1)).max(0.0)
    }

    fn union(box1: &BoundingBox, box2: &BoundingBox) -> f32 {
        (box1.x2 - box1.x1) * (box1.y2 - box1.y1) + (box2.x2 - box2.x1) * (box2.y2 - box2.y1)
            - Self::intersection(box1, box2)
    }
}
