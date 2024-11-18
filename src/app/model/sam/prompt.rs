#[derive(Debug, Clone, Copy)]
pub enum Prompt {
    Point(([f32; 2], f32)), // point, label
    Box([f32; 4]),          // box, left-top x, left-top y, right-bottom x, right-bottom y
}

impl Prompt {
    pub fn new_point(x: f32, y: f32, label: f32) -> Self {
        assert!(label == 0.0 || label == 1.0);
        assert!(x >= 0.0 && x <= 1.0);
        assert!(y >= 0.0 && y <= 1.0);

        Self::Point(([x, y], label))
    }

    pub fn new_box(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        assert!(x1 < x2 && y1 < y2);
        assert!(x1 >= 0.0 && x2 <= 1.0);
        assert!(y1 >= 0.0 && y2 <= 1.0);

        Self::Box([x1, y1, x2, y2])
    }
}

impl From<crate::app::model::yolo::BoundingBox> for Prompt {
    fn from(bb: crate::app::model::yolo::BoundingBox) -> Self {
        Prompt::new_box(bb.x1, bb.y1, bb.x2, bb.y2)
    }
}

impl From<[f32; 4]> for Prompt {
    fn from(bb: [f32; 4]) -> Self {
        Prompt::new_box(bb[0], bb[1], bb[2], bb[3])
    }
}

impl From<([f32; 2], f32)> for Prompt {
    fn from(point: ([f32; 2], f32)) -> Self {
        let ([x, y], label) = point;
        Prompt::new_point(x, y, label)
    }
}

impl From<(&[f32; 2], &f32)> for Prompt {
    fn from(data: (&[f32; 2], &f32)) -> Self {
        let ([x, y], label) = data;
        Prompt::new_point(*x, *y, *label)
    }
}

impl Into<(Vec<f32>, Vec<f32>)> for Prompt {
    fn into(self) -> (Vec<f32>, Vec<f32>) {
        match self {
            Prompt::Point((point, label)) => (vec![point[0], point[1]], vec![label]),
            Prompt::Box(bb) => {
                let [x1, y1, x2, y2] = bb;

                (vec![x1, y1, x2, y2], vec![2.0, 3.0])
            }
        }
    }
}
