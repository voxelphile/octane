mod window;

mod term {
    pub const RESET: &str = "\x1b[1;0m";
    pub const BOLDMAGENTA: &str = "\x1b[1;35m";
    pub const BOLDCYAN: &str = "\x1b[1;36m";
    pub const BOLDYELLOW: &str = "\x1b[1;33m";
    pub const BOLDRED: &str = "\x1b[1;31m";
}

use crate::window::{Event as WindowEvent, Keycode, Window};

use common::mesh::Mesh;
use common::render::{self, Renderer};

use math::prelude::{Matrix, Vector};

use std::collections::HashMap;
use std::fs;
use std::mem;
use std::path::Path;

use log::{error, info, trace, warn};

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Trace
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!(
                "{}{}{}: {}",
                match record.level() {
                    log::Level::Trace => term::BOLDMAGENTA,
                    log::Level::Info => term::BOLDCYAN,
                    log::Level::Warn => term::BOLDYELLOW,
                    log::Level::Error => term::BOLDRED,
                    _ => term::RESET,
                },
                record.level().as_str().to_lowercase(),
                term::RESET,
                record.args()
            );
        }
    }
    fn flush(&self) {}
}

static LOGGER: Logger = Logger;

//TODO identify why release segfaults
fn main() {
    println!("Hello, world!");

    log::set_max_level(log::LevelFilter::Info);
    log::set_logger(&LOGGER).expect("failed to set logger");

    let mut window = Window::new();

    window.rename("Octane");
    window.show();

    let mut projection = Matrix::<f32, 4, 4>::identity();
    let mut camera = Matrix::<f32, 4, 4>::identity();
    let mut model = Matrix::<f32, 4, 4>::identity();

    //create matrices
    {
        let fov = 45.0_f32 * 2.0 * std::f32::consts::PI / 360.0;

        let focal_length = 1.0 / (fov / 2.0).tan();

        let aspect_ratio = (960) as f32 / (540) as f32;

        let near = 0.01;
        let far = 1000.0;

        projection[0][0] = focal_length / aspect_ratio;
        projection[1][1] = -focal_length;
        projection[2][2] = far / (near - far);
        projection[2][3] = -1.0;
        projection[3][2] = (near * far) / (near - far);
    }

    let mut base_path = std::env::current_exe().expect("failed to load path of executable");
    base_path.pop();
    let base_path_str = base_path.to_str().unwrap();

    let render_distance = 2;

    let hq4x = format!("{}/assets/hq4x.png", base_path_str);

    let render_info = render::RendererInfo {
        window: &window,
        render_distance,
        hq4x,
    };

    let mut vulkan = render::Vulkan::init(render_info);

    vulkan.camera.proj = projection;
    vulkan.camera.view = camera.inverse();
    vulkan.camera.camera = camera;
    vulkan.camera.model = model;

    let present_vertex_shader = format!("{}/assets/fullscreen.vert.spirv", base_path_str);
    let present_fragment_shader = format!("{}/assets/present.frag.spirv", base_path_str);
    let postfx_vertex_shader = format!("{}/assets/fullscreen.vert.spirv", base_path_str);
    let postfx_fragment_shader = format!("{}/assets/postfx.frag.spirv", base_path_str);
    let graphics_vertex_shader = format!("{}/assets/default.vert.spirv", base_path_str);
    let graphics_fragment_shader = format!("{}/assets/default.frag.spirv", base_path_str);
    let jfa_shader = format!("{}/assets/jfa.comp.spirv", base_path_str);

    let cube = format!("{}/assets/cube.obj", base_path_str);
    let cube_obj = fs::File::open(cube).expect("failed to open obj");

    let mut cube = Mesh::from_obj(cube_obj);

    let batch = render::Batch {
        graphics_vertex_shader,
        graphics_fragment_shader,
        postfx_vertex_shader,
        postfx_fragment_shader,
        present_vertex_shader,
        present_fragment_shader,
        jfa_shader,
    };

    let entries = [render::Entry { mesh: &cube }];

    let startup = std::time::Instant::now();
    let mut last = startup;

    let mut keys = HashMap::new();

    let mut x_rot = 0.0;
    let mut y_rot = 0.0;
    let middle = (2.0 * render_distance as f32 * 8.0) / 2.0 - 4.0;
    let height = 32.0;
    let mut position = Vector::<f32, 4>::new([middle, height, middle, 1.0]);
    let mut should_capture = false;

    let mut fps_instant = startup;
    let mut fps = 0;

    'main: loop {
        let current = std::time::Instant::now();
        let delta_time = current.duration_since(last).as_secs_f64();
        last = current;

        if current.duration_since(fps_instant).as_secs_f32() > 1.0 {
            window.rename(format!("Octane {}", fps).as_str());
            fps_instant = current;
            fps = 0;
        }

        if should_capture {
            window.capture();
        }
        window.show_cursor(!should_capture);

        //TODO must be done soon.. tired of this convoluted movement code.
        //simplify movement code
        let sens = 2.0;

        while let Some(event) = window.next_event() {
            match event {
                WindowEvent::KeyPress { keycode } => {
                    if should_capture || keycode == crate::window::Keycode::Escape {
                        keys.insert(keycode, current);
                    }
                }
                WindowEvent::KeyRelease { keycode } => {
                    keys.remove(&keycode);
                }
                WindowEvent::PointerMotion { x, y } => {
                    if should_capture {
                        x_rot -= ((x as f64 - window.resolution().0 as f64 / 2.0) * delta_time)
                            as f32
                            / sens;
                        y_rot -= ((y as f64 - window.resolution().1 as f64 / 2.0) * delta_time)
                            as f32
                            / sens;
                    }
                }
                WindowEvent::CloseRequested => {
                    break 'main;
                }
                WindowEvent::Resized { resolution } => {
                    {
                        let fov = 45.0_f32 * 2.0 * std::f32::consts::PI / 360.0;

                        let focal_length = 1.0 / (fov / 2.0).tan();

                        let aspect_ratio = resolution.0 as f32 / resolution.1 as f32;

                        let near = 0.01;
                        let far = 1000.0;

                        projection[0][0] = focal_length / aspect_ratio;
                        projection[1][1] = -focal_length;
                        projection[2][2] = far / (near - far);
                        projection[2][3] = -1.0;
                        projection[3][2] = (near * far) / (near - far);
                    }
                    vulkan.resize(resolution);
                }
            }
        }

        let movement_speed = 10.92;

        let mut camera = Matrix::<f32, 4, 4>::identity();

        let mut x_r = Matrix::<f32, 4, 4>::identity();
        let mut y_r = Matrix::<f32, 4, 4>::identity();

        x_r[0][0] = x_rot.cos();
        x_r[2][0] = x_rot.sin();
        x_r[0][2] = -x_rot.sin();
        x_r[2][2] = x_rot.cos();

        y_rot = y_rot.clamp(
            -std::f32::consts::PI / 2.0 + 0.1,
            std::f32::consts::PI / 2.0 - 0.1,
        );

        y_r[1][1] = y_rot.cos();
        y_r[2][1] = -y_rot.sin();
        y_r[1][2] = y_rot.sin();
        y_r[2][2] = y_rot.cos();

        camera = camera * y_r;
        camera = camera * x_r;

        let mut m = Matrix::<f32, 4, 4>::identity();

        for (key, &time) in &keys {
            match key {
                Keycode::W => {
                    m[3][2] += -1.0;
                }
                Keycode::A => {
                    m[3][0] += -1.0;
                }
                Keycode::S => {
                    m[3][2] += 1.0;
                }
                Keycode::D => {
                    m[3][0] += 1.0;
                }
                Keycode::Space => {
                    position[1] += movement_speed * delta_time as f32;
                }
                Keycode::LeftShift => {
                    position[1] -= movement_speed * delta_time as f32;
                }
                Keycode::Escape => {
                    if time == current {
                        should_capture = !should_capture;
                    }
                }
            }
        }

        let l = m * y_r;
        let l = l * x_r;
        let mut p = Vector::<f32, 4>::new(l[3]);
        p[1] = 0.0;
        p[3] = 0.0;
        let p = if p.magnitude() > 0.0 {
            p.normalize()
        } else {
            p
        };
        position[0] += p[0] * movement_speed * delta_time as f32;
        position[2] += p[2] * movement_speed * delta_time as f32;

        camera[3][0] = position[0];
        camera[3][1] = position[1];
        camera[3][2] = position[2];

        let follow = 2.0 * 1.0 as f32;
        let angle: f32 = 0.0;

        vulkan.camera.model = Matrix::<f32, 4, 4>::identity();
        vulkan.camera.model[0][0] = angle.cos();
        vulkan.camera.model[2][0] = angle.sin();
        vulkan.camera.model[0][2] = -angle.sin();
        vulkan.camera.model[2][2] = angle.cos();

        vulkan.camera.view = camera.inverse();

        vulkan.camera.camera = camera;

        vulkan.draw_batch(batch.clone(), &entries);

        fps += 1;
    }

    //TODO figure out surface dependency on window
    //window is dropped before surface which causes segfault
    //explicit drop fixes this but it is not ideal

    drop(vulkan);
    drop(window);
    //vk shutdown happens during implicit Drop.
    //Rc ensures shutdown happens in right order.
}
