use macroquad::color::hsl_to_rgb;
use macroquad::prelude::*;
use nalgebra::{Matrix3, Vector2};

#[macroquad::main("Dragon Curve")]
async fn main() {
    let mut folds: usize = 6;
    let mut angle_delta_per_second = 0.0;
    let mut angle = std::f32::consts::FRAC_PI_2;
    let mut dragon = Dragon::new();

    loop {
        clear_background(BLACK);

        if is_key_pressed(KeyCode::Up) {
            folds += 1;
        }
        if is_key_pressed(KeyCode::Down) {
            folds = folds.saturating_sub(1);
        }

        if is_key_pressed(KeyCode::Left) {
            angle_delta_per_second -= 0.01;
        }
        if is_key_pressed(KeyCode::Right) {
            angle_delta_per_second += 0.01;
        }

        if is_key_pressed(KeyCode::Q) {
            break;
        }

        angle += angle_delta_per_second * get_frame_time();

        dragon.curve(folds, angle);
        let width = screen_width();
        let height = screen_height();
        let transform = to_screenspace(&dragon.vertices, width, height);

        for (i, (a, b)) in dragon
            .vertices
            .iter()
            .zip(dragon.vertices.iter().skip(1))
            .enumerate()
        {
            let a = transform.transform_point(&(*a).into());
            let b = transform.transform_point(&(*b).into());
            let along = i as f32 / (dragon.vertices.len() - 1) as f32;
            let color = hsl_to_rgb(along, 0.5, 0.5);
            draw_line(a.x, a.y, b.x, b.y, 2.0, color);
        }

        next_frame().await;
    }
}

/// determine the transformation that would center the curve in the screen, keeping it from entering a 5% margin
fn to_screenspace(dragon: &[Vector2<f32>], width: f32, height: f32) -> Matrix3<f32> {
    let dragon_center = dragon
        .iter()
        .fold(Vector2::new(0.0, 0.0), |acc, v| acc + *v)
        / dragon.len() as f32;

    let max_x = dragon
        .iter()
        .map(|v| v - dragon_center)
        .map(|v| v.x.abs())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    let max_y = dragon
        .iter()
        .map(|v| v - dragon_center)
        .map(|v| v.y.abs())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    let x_bound = width * 0.95 * 0.5;
    let y_bound = height * 0.95 * 0.5;
    let scale = (x_bound / max_x).min(y_bound / max_y);

    let a = Matrix3::new_translation(&-dragon_center);
    let b = Matrix3::new_nonuniform_scaling(&Vector2::new(scale, scale));
    let c = Matrix3::new_translation(&Vector2::new(width / 2.0, height / 2.0));

    c * b * a
}

struct Dragon {
    vertices: Vec<Vector2<f32>>,
}

impl Dragon {
    fn curve(&mut self, folds: usize, angle: f32) {
        self.vertices.clear();
        self.vertices.push(Vector2::new(0.0, 0.0));
        self.vertices.push(Vector2::new(1.0, 0.0));
        for _ in 0..folds {
            self.double(angle);
        }
    }

    fn new() -> Self {
        Self {
            vertices: [Vector2::new(0.0, 0.0), Vector2::new(1.0, 0.0)].to_vec(),
        }
    }

    fn double(&mut self, angle: f32) {
        assert!(!self.vertices.is_empty());

        let start_len = self.vertices.len();
        debug_assert!(start_len % 2 == 1 || start_len == 2);

        let pivot = self.vertices[start_len - 1];

        let a_translate = Matrix3::new_translation(&-pivot);
        let b_rotate = Matrix3::new_rotation(angle);
        let c_translate = Matrix3::new_translation(&pivot);
        let transform = c_translate * b_rotate * a_translate;

        // fill vec with default values so its length is 2 * origin.len() - 1
        self.vertices
            .resize(2 * self.vertices.len() - 1, Vector2::new(0.0, 0.0));

        // split the old from the new, we will use the old to determine new
        let (old, new) = self.vertices.split_at_mut(start_len);
        for (old, new) in old.iter().rev().skip(1).zip(new.iter_mut()) {
            let next = transform.transform_point(&(*old).into());
            *new = Vector2::new(next.x, next.y);
        }
    }
}
