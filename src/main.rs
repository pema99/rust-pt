#[macro_use]
extern crate glium;

use cgmath::*;

pub mod trace;
use trace::*;

fn main() {
    let ctx = Context::new();
    let scene = Scene { 
        cam_pos: Vector3::new(0.0, 0.0, -1.0),
        cam_rot: Quaternion::one()
    };
    let cfg = Config {
        width: 100,
        height: 100,
        min_bounces: 0,
        max_bounces: 3,
        samples_per_pixel: 1
    };

    render(&ctx, &cfg, &scene);
}
