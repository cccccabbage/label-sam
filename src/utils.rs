use geo::{LineString, Simplify};
use image::GrayImage;
use imageproc::{
    contours::{find_contours, Contour},
    distance_transform::Norm,
};

#[allow(dead_code)]
pub fn mask_or(masks: Vec<image::GrayImage>) -> image::GrayImage {
    assert!(!masks.is_empty());

    let mut result = masks[0].clone();
    for m in masks.into_iter().skip(1) {
        result = imageproc::map::map_colors2(&result, &m, |p, q| image::Luma([p[0] | q[0]]));
    }

    result
}

pub fn extract_outline(mask: &GrayImage) -> Vec<[f32; 2]> {
    // morphological opening, it can remove small object and noise
    // it's not the same as erosion
    let mask = imageproc::morphology::open(mask, Norm::LInf, 3);

    let contours: Vec<Contour<i32>> = find_contours(&mask); // find all contours
    let contour = contours.into_iter().max_by_key(|c| c.points.len()).unwrap(); // keep the longest one

    const EPSILON: f64 = 2.0;

    // change to LineString type so than we can simplify it
    let line_string: LineString<f64> = contour
        .points
        .iter()
        .map(|p| (p.x as f64, p.y as f64))
        .collect();

    // the simplification function uses the Ramer-Douglas-Peucker algorithm
    let simplified_line = line_string.simplify(&EPSILON);

    simplified_line // return to Vec<[f32;2]>
        .coords()
        .map(|p| [p.x as f32, p.y as f32])
        .collect()
}
