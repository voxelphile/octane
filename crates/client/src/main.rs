mod window;

mod term {
    pub const RESET: &str = "\x1b[1;0m";
    pub const BOLDMAGENTA: &str = "\x1b[1;35m";
    pub const BOLDCYAN: &str = "\x1b[1;36m";
    pub const BOLDYELLOW: &str = "\x1b[1;33m";
    pub const BOLDRED: &str = "\x1b[1;31m";
}

use crate::window::{Event as WindowEvent, Window};

use common::mesh::Mesh;
use common::render::{self, Renderer};

use math::prelude::Matrix;

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

    let mut vulkan = render::Vulkan::init(&window);

    vulkan.ubo.proj = projection;
    vulkan.ubo.view = camera.inverse();
    vulkan.ubo.model = model;

    let vertex_shader = "/home/brynn/dev/octane/assets/default.vs.spirv";
    let fragment_shader = "/home/brynn/dev/octane/assets/default.fs.spirv";

    let cube_obj =
        fs::File::open("/home/brynn/dev/octane/assets/cube.obj").expect("failed to open obj");

    let mut cube = Mesh::from_obj(cube_obj);

    let batch = render::Batch {
        vertex_shader: &vertex_shader,
        fragment_shader: &fragment_shader,
    };

    let entries = [render::Entry { mesh: &cube }];

    let startup = std::time::Instant::now();

    loop {
        let event = window.next_event();

        match event {
            Some(WindowEvent::CloseRequested) => {
                break;
            }
            Some(WindowEvent::Resized { resolution }) => {
                dbg!("resized");
                vulkan.resize(resolution);
            }
            None => {}
            _ => {}
        }

        let follow = 2.0 * 1.0 as f32;
        let angle = std::time::Instant::now()
            .duration_since(startup)
            .as_secs_f32()
            % (2.0 * std::f32::consts::PI);

        vulkan.ubo.model[0][0] = angle.cos();
        vulkan.ubo.model[2][0] = angle.sin();
        vulkan.ubo.model[0][2] = -angle.sin();
        vulkan.ubo.model[2][2] = angle.cos();

        camera[3][2] = 3.0;

        vulkan.ubo.view = camera.inverse();

        vulkan.draw_batch(batch.clone(), &entries);
    }

    //TODO figure out surface dependency on window
    //window is dropped before surface which causes segfault
    //explicit drop fixes this but it is not ideal

    drop(vulkan);
    drop(window);
    //vk shutdown happens during implicit Drop.
    //Rc ensures shutdown happens in right order.
}
