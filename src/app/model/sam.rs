pub mod prompt;

use prompt::Prompt;

use lazy_static::lazy_static;
use ndarray::{Array2, Array3, Array4, ArrayBase, Dim, IxDynImpl, ViewRepr};
use ort::{inputs, CUDAExecutionProvider, GraphOptimizationLevel, Session};

use image::{DynamicImage, GenericImageView};

const INPUT_W: u32 = 1024;
const INPUT_H: u32 = 684;

lazy_static! {
    static ref MASK: ndarray::Array4<f32> = ndarray::Array4::<f32>::default((1, 1, 256, 256));
    static ref HAS_MASK_INPUT: ndarray::Array1<f32> = ndarray::Array1::from(vec![0.0f32]);
    static ref ORIG_SIZE: ndarray::Array1<f32> =
        ndarray::Array1::from(vec![INPUT_H as f32, INPUT_W as f32]);
}

#[derive(Debug)]
pub struct SAMmodel {
    encoder: Session,
    decoder: Session,

    embedding: Option<Array4<f32>>,
    ori_w: u32,
    ori_h: u32,
}

impl SAMmodel {
    pub fn new() -> Self {
        Self::new_path("weights/sam_b-encoder.onnx", "weights/sam_b-decoder.onnx")
    }

    pub fn new_path(encoder_path: &str, decoder_path: &str) -> Self {
        let encoder = Session::builder()
            .unwrap()
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .unwrap()
            .with_execution_providers([CUDAExecutionProvider::default().build().error_on_failure()])
            .unwrap()
            .with_intra_threads(4)
            .unwrap()
            .commit_from_file(encoder_path)
            .unwrap();

        let decoder = Session::builder()
            .unwrap()
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .unwrap()
            .with_execution_providers([CUDAExecutionProvider::default().build().error_on_failure()])
            .unwrap()
            .with_intra_threads(4)
            .unwrap()
            .commit_from_file(decoder_path)
            .unwrap();

        Self {
            encoder,
            decoder,
            embedding: None,
            ori_w: 0,
            ori_h: 0,
        }
    }

    pub fn forward(&mut self, img: &DynamicImage, prompt: Prompt) -> DynamicImage {
        self.embed(img).unwrap();
        self.generate_mask(prompt)
    }

    pub fn embed(&mut self, img: &DynamicImage) -> Result<(), Box<dyn std::error::Error>> {
        let (embedding, w, h) = Self::preprocess_img(img);
        self.ori_w = w;
        self.ori_h = h;

        let encoder_input = inputs!(&self.encoder.inputs[0].name => embedding.view())?;
        let mut encoder_output = self.encoder.run(encoder_input)?;
        self.embedding = Some(
            encoder_output
                .remove("image_embeddings")
                .unwrap()
                .try_extract_tensor::<f32>()?
                .to_shape((1, 256, 64, 64))?
                .to_owned(),
        );

        Ok(())
    }

    // the prompts should be normalized
    pub fn generate_mask(&self, prompt: Prompt) -> DynamicImage {
        let (points, labels) = Self::preprocess_prompts(prompt);
        let emb = self.embedding.as_ref().unwrap();
        let decoder_input = inputs!(
            &self.decoder.inputs[0].name => emb.view(),
            &self.decoder.inputs[1].name => points.view(),
            &self.decoder.inputs[2].name => labels.view(),
            &self.decoder.inputs[3].name => MASK.view(),
            &self.decoder.inputs[4].name => HAS_MASK_INPUT.view(),
            &self.decoder.inputs[5].name => ORIG_SIZE.view(),
        )
        .unwrap();
        let decoder_output = self.decoder.run(decoder_input).unwrap();
        let output = decoder_output["masks"].try_extract_tensor::<f32>().unwrap();
        Self::postprocess(output, self.ori_w, self.ori_h)
    }

    fn preprocess_img(img: &DynamicImage) -> (Array3<f32>, u32, u32) {
        let (ori_w, ori_h) = img.dimensions();
        let img = img.resize_exact(INPUT_W, INPUT_H, image::imageops::FilterType::CatmullRom);

        let mut arr = Array3::zeros((INPUT_H as usize, INPUT_W as usize, 3));
        for pixel in img.pixels() {
            let x = pixel.0 as _;
            let y = pixel.1 as _;
            let [r, g, b, _] = pixel.2 .0;

            arr[[y, x, 2]] = r as f32;
            arr[[y, x, 1]] = g as f32;
            arr[[y, x, 0]] = b as f32;
        }

        (arr, ori_w, ori_h)
    }

    fn preprocess_prompts(prompt: Prompt) -> (Array3<f32>, Array2<f32>) {
        let (points, labels): (Vec<f32>, Vec<f32>) = prompt.clone().into();
        let points = points
            .chunks(2) // make points to Vec<(f32, f32)>
            .flat_map(|chunk| {
                // each chunk is (f32, f32)
                let x = chunk[0] * INPUT_W as f32;
                let y = chunk[1] * INPUT_H as f32;
                vec![x, y] // returns Vec<f32>
            })
            .collect(); // collect to Vec<f32>

        let points = Array3::from_shape_vec((1, labels.len(), 2), points).unwrap();

        let labels = Array2::from_shape_vec((1, labels.len()), labels.clone()).unwrap();

        (points, labels)
    }

    fn postprocess(
        mask: ArrayBase<ViewRepr<&f32>, Dim<IxDynImpl>>,
        w: u32,
        h: u32,
    ) -> DynamicImage {
        let mask = mask.mapv(|v| if v > 0.5f32 { 255u8 } else { 0u8 });
        let mask: Vec<u8> = mask.flatten().to_vec();
        image::DynamicImage::ImageLuma8(image::GrayImage::from_raw(INPUT_W, INPUT_H, mask).unwrap())
            .resize_exact(w, h, image::imageops::FilterType::CatmullRom)
    }
}
