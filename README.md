# Label-SAM

## Intro
This project is a simple application to generate segmentation masks for images.
It is based on the [YOLOv8](https://docs.ultralytics.com/models/yolov8/) and [SAM](https://github.com/facebookresearch/segment-anything).

One can easily use the model to generate segmentation masks of different instances in an image,
and then output to a txt file with yolo-seg format.

## How to use
First you need to prepare an onnx model of yolo, you can check [this](https://docs.ultralytics.com/modes/export/) to learn about acquire an onnx model for your own yolo model.

Then you need to export SAM to onnx, which is based on the [samexporter](https://github.com/vietanhdev/samexporter) in this project.

I haven't add the functionality to import custom model so you may need to edit the paths in config.json to import your own model.

``` json
{
  "yolo_path": "weights/yolov8s-trained.onnx",
  "sam_e_path": "weights/sam_b-encoder.onnx",
  "sam_d_path": "weights/sam_b-decoder.onnx"
}
```

And I haven't tested execution providers other than cuda, so you should check it yourself.
See [this](https://ort.pyke.io/perf/execution-providers) for help.

``` rust
// src/app/model/yolo.rs and src/app/model/sam.rs
.with_execution_providers([CUDAExecutionProvider::default().build()])
```
