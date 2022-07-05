use cgmath::*;
use glium::backend::glutin::headless::Headless;
use glium::program::ComputeShader;
use glium::uniforms::UniformBuffer;
use glutin::dpi::PhysicalSize;
use image::{ImageBuffer, RgbImage};

fn import_shader(display: &Headless, path: &str) -> ComputeShader {
    let code = std::fs::read_to_string(path).expect("Failed to load shader");
    ComputeShader::from_source(display, &code).expect("Failed to compile shader")
}

pub struct Context {
    display: Headless,
    cs_intersect: ComputeShader
}

impl Context {
    pub fn new() -> Self {
        // Setup
        let event_loop = glium::glutin::event_loop::EventLoop::new();
        let cb = glutin::ContextBuilder::new();
        let size = PhysicalSize {
            width: 800,
            height: 600,
        };
        let context = cb.build_headless(&event_loop, size).unwrap();
        let context = unsafe { context.treat_as_current() };
        let display = glium::backend::glutin::headless::Headless::new(context).unwrap();

        // Compile kernels
        let cs_intersect = import_shader(&display, "src/kernels/intersect.glsl");

        // Build context
        Self {
            display,
            cs_intersect
        }
    }
}

pub struct Scene {
    pub cam_pos: Vector3<f32>,
    pub cam_rot: Quaternion<f32>,
}

pub struct Config {
    pub width: u32,
    pub height: u32,
    pub min_bounces: u32,
    pub max_bounces: u32,
    pub samples_per_pixel: u32,
}

pub fn render(ctx: &Context, cfg: &Config, scene: &Scene) {
    let mut ray_dirs = Vec::new();
    ray_dirs.reserve((cfg.width * cfg.height) as usize);
    for y in 0..cfg.width {
        for x in 0..cfg.height {
            let rx = x as f32 / cfg.width as f32;
            let ry = y as f32 / cfg.height as f32;
            let ray_dir = Vector3::new((rx - 0.5) * 2.0, (ry - 0.5) * 2.0, 1.0).normalize();
            ray_dirs.push(ray_dir);
        }
    }
    
    // set buffers
    let mut buffer: UniformBuffer<[[f32; 4]]> =
        UniformBuffer::empty_unsized(&ctx.display, ray_dirs.len()*4*4).unwrap();

    //let mut buffer: UniformBuffer<Data> = UniformBuffer::empty(&ctx.display).unwrap();

    {
        let mut mapping = buffer.map();
        for i in 0..ray_dirs.len() {
            let ray_dir = ray_dirs[i];
            mapping[i] = [ray_dir.x, ray_dir.y, ray_dir.z, -1.0];
        }
    }

    ctx.cs_intersect.execute(uniform! { RayDirections: &*buffer }, ray_dirs.len() as u32, 1, 1);

    // calculate intersection
    // calculate shading

    {
        let mapping = buffer.map();
        let img: RgbImage = ImageBuffer::from_fn(cfg.width, cfg.height, |x, y| {
            let col = mapping[(y * cfg.width + x) as usize];
            image::Rgb([(col[0]*255.0) as u8, (col[1]*255.0)as u8, (col[2]*255.0) as u8])
        });
        img.save("out.png").expect("Failed to save image");
    }
}

fn trace(scene: Scene, x: usize, y: usize, width: usize, height: usize) {}
