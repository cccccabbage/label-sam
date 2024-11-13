pub fn mask_or(masks: Vec<image::GrayImage>) -> image::GrayImage {
    assert!(!masks.is_empty());

    let mut result = masks[0].clone();
    for m in masks.iter().skip(1) {
        result = imageproc::map::map_colors2(&result, m, |p, q| image::Luma([p[0] | q[0]]));
    }

    result
}
