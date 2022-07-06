use std::rc::Rc;

use cgmath::*;
use glium::texture::{UncompressedFloatFormat, MipmapsOption};
use glium::{Display, Surface, Program, Texture2d};
use glium::program::ComputeShader;
use glium::uniforms::UniformBuffer;
use glutin::event_loop::EventLoop;
use image::{ImageBuffer, RgbImage};

fn import_kernel(display: &Display, path: &str) -> ComputeShader {
    let code = std::fs::read_to_string(path).expect("Failed to load shader");
    ComputeShader::from_source(display, &code).expect("Failed to compile shader")
}

fn import_vert_frag(display: &Display, path_vert: &str, path_frag: &str) -> Program {
    let code_vert = std::fs::read_to_string(path_vert).expect("Failed to vertex shader");
    let code_frag = std::fs::read_to_string(path_frag).expect("Failed to fragment shader");
    glium::Program::from_source(display, &code_vert, &code_frag, None).unwrap()
}

pub struct Context {
    display: Display,
    cs_intersect: ComputeShader
}

impl Context {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        // Setup
        let wb = glutin::window::WindowBuilder::new();
        let cb = glutin::ContextBuilder::new();
        
        let display = glium::Display::new(wb, cb, event_loop).unwrap();

        // Compile kernels
        let cs_intersect = import_kernel(&display, "src/kernels/intersect.glsl");

        // Build context
        Self {
            display,
            cs_intersect
        }
    }
}

pub fn main_loop() {
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let ctx = Context::new(&event_loop);

    let scene = Scene { 
        cam_pos: Vector3::new(0.0, 0.0, -1.0),
        cam_rot: Quaternion::one()
    };
    let cfg = Config {
        width: 1024,
        height: 1024,
        min_bounces: 0,
        max_bounces: 3,
        samples_per_pixel: 1
    };

    // Setup vert buffer
    #[derive(Copy, Clone)]
    struct Vertex { position: [f32; 2] }
    implement_vertex!(Vertex, position);

    let vertex1 = Vertex { position: [-1.0, -1.0] };
    let vertex2 = Vertex { position: [-1.0,  1.0] };
    let vertex3 = Vertex { position: [ 1.0, -1.0] };
    let vertex4 = Vertex { position: [ 1.0,  1.0] };
    let shape = vec![vertex1, vertex2, vertex3, vertex4, vertex2, vertex3];

    let vertex_buffer = glium::VertexBuffer::new(&ctx.display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    // Compiler shader
    let program = import_vert_frag(&ctx.display, "src/shaders/basic_vert.glsl", "src/shaders/basic_frag.glsl");

    let framebuffer = Texture2d::empty_with_format(&ctx.display,
        UncompressedFloatFormat::U8U8U8,
        MipmapsOption::NoMipmap,
        cfg.width,
        cfg.height).unwrap();
    framebuffer.as_surface().clear_color(1.0, 0.0, 0.0, 1.0);
    render(&ctx, &cfg, &scene, &framebuffer);

    // Main event loop
    event_loop.run(move |event, _, control_flow| {
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                glutin::event::WindowEvent::Resized(_) => {
                    return;
                }
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }

        let next_frame_time = std::time::Instant::now() +
        std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        // Draw current result
        let mut target = ctx.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(&vertex_buffer, &indices, &program, &uniform! { framebuffer: &framebuffer }, &Default::default()).unwrap();
        target.finish().unwrap();
    })
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

pub fn render(ctx: &Context, cfg: &Config, scene: &Scene, tar: &Texture2d) {
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
        let img = glium::texture::RawImage2d::from_raw_rgb(img.into_raw(), (cfg.width, cfg.height));
        tar.write(glium::Rect { left: 0, bottom: 0, width: cfg.width, height: cfg.height }, img);
    }
}
