#![feature(let_else)]
#![feature(box_syntax)]

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
use common::octree::{Octree, SparseOctree};
use common::render::{self, Condition, Renderer};
use common::voxel::{Id::*, Voxel};
use common::bitfield::*;

//use input::prelude::*;
use math::prelude::{Matrix, Vector};

use std::collections::HashMap;
use std::error::Error;
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
            print!(
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

pub const CHUNK_SIZE: usize = 8;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");

    log::set_max_level(log::LevelFilter::Info);
    log::set_logger(&LOGGER).expect("failed to set logger");

    let mut window = Window::new();

    window.rename("Octane");
    window.show();

    window.fullscreen(false);

    let render_distance = 32;

    let mut octree = {

        /*
        let ct = 2 * render_distance as usize * CHUNK_SIZE;

        use noise::NoiseFn;
        let perlin = noise::Perlin::new();

        for x in 0..ct {
            for z in 0..ct {
                let mut max_y = 16.0 as isize;
                for o in 1..=4 {
                    max_y += ((5.0 as f64 / (o as f64).powf(0.5))
                        * perlin.get([x as f64 / (o as f64 * 32.0), z as f64 / (o as f64 * 32.0)]))
                        as isize;
                }
                for y in 0..ct {
                    if y >= max_y as usize && y < 16 {
                        octree.place(x, y, z, Voxel { id: Water });
                    } else if y == max_y as usize - 1 {
                        octree.place(x, y, z, Voxel { id: Grass });
                    } else if y < max_y as usize {
                        octree.place(x, y, z, Voxel { id: Dirt });
                    }
                }
            }
            print!(
                "\r{}info{}: Building octree: {}%",
                term::BOLDCYAN,
                term::RESET,
                ((x as f32 / ct as f32) * 100.0) as usize
            );
        }

        print!(
            "\r{}info{}: Building octree: {}%\n",
            term::BOLDCYAN,
            term::RESET,
            100
        );
        info!("Optimizing octree\n");
        octree.optimize();

        octree
        */
        let mut octree = SparseOctree::<Voxel>::new();
        for x in 0..=0 {
            for y in 0..=0 {
                for z in 0..=0 {
                    octree.place(1,1,1, Voxel { id: Dirt });
                }
            }
        }
        octree
    };
    octree.optimize();
    let bitfield = octree.build_bitfield();

    dbg!(&bitfield);
    dbg!(&bitfield.data());


    panic!("");

    //create matrices

    let mut base_path = std::env::current_exe().expect("failed to load path of executable");
    base_path.pop();
    let base_path_str = base_path.to_str().unwrap();
    let base_path_str = base_path_str.replace("\\\\?\\", "");

    let hq4x = format!("{}\\assets\\hq4x.png", base_path_str);

    let render_info = render::RendererInfo {
        window: &window,
        render_distance,
        hq4x,
    };

    let mut vulkan = render::Vulkan::init(render_info);

    let startup = std::time::Instant::now();
    let mut last = startup;

    let mut keys = HashMap::new();

    let mut x_rot = 0.0;
    let mut y_rot = 0.0;
    let middle = (2.0 * render_distance as f32 * 8.0) / 2.0 - 4.0;
    let height = 16.0;
    let mut position = Vector::<f32, 4>::new([middle, height, middle, 1.0]);
    let mut should_capture = false;
    let mut prev_should_capture = false;
    let mut focus_lost = true;

    let mut fps_instant = startup;
    let mut fps = 0;

    let mut camera = render::Camera::default();

    camera.proj = {
        let mut projection = Matrix::<f32, 4, 4>::identity();

        let fov = 45.0_f32 * 2.0 * std::f32::consts::PI / 360.0;

        let focal_length = 1.0 / (fov / 2.0).tan();

        let aspect_ratio = 960 as f32 / 540 as f32;

        let near = 0.01;
        let far = 1000.0;

        projection[0][0] = focal_length / aspect_ratio;
        projection[1][1] = -focal_length;
        projection[2][2] = far / (near - far);
        projection[2][3] = -1.0;
        projection[3][2] = (near * far) / (near - far);
        projection
    };

    'main: loop {
        let current = std::time::Instant::now();
        let delta_time = current.duration_since(last).as_secs_f64();
        last = current;

        if current.duration_since(fps_instant).as_secs_f32() > 1.0 {
            window.rename(format!("Octane {}", fps).as_str());
            fps_instant = current;
            fps = 0;
        }

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
                        let (mut cx, mut cy) = window.resolution();
                        cx /= 2;
                        cy /= 2;
                        let difx = (x as f64 - cx as f64);
                        let dify = (y as f64 - cy as f64);
                        x_rot -= (difx * delta_time) as f32 / sens;
                        y_rot -= (dify * delta_time) as f32 / sens;
                    }
                }
                WindowEvent::FocusIn => {
                    if focus_lost {
                        should_capture = prev_should_capture;
                        focus_lost = false;
                    }
                }
                WindowEvent::FocusOut => {
                    if !focus_lost {
                        prev_should_capture = should_capture;
                        should_capture = false;
                        focus_lost = true;
                        keys.clear();
                    }
                }
                WindowEvent::CloseRequested => {
                    break 'main;
                }
                WindowEvent::Resized { resolution } => {
                    camera.proj = {
                        let mut projection = Matrix::<f32, 4, 4>::identity();

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
                        projection
                    };

                    vulkan.resize(resolution);
                }
            }
        }

        if should_capture {
            window.capture();
        }
        window.show_cursor(!should_capture);

        let movement_speed = 10.92;

        camera.model = Matrix::identity();
        camera.view = Matrix::identity();

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

        camera.model = camera.model * y_r;
        camera.model = camera.model * x_r;

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
        let mut p = Vector::<f32, 4>::new(*l[3]);
        p[1] = 0.0;
        p[3] = 0.0;
        let p = if p.magnitude() > 0.0 {
            p.normalize()
        } else {
            p
        };
        position[0] += p[0] * movement_speed * delta_time as f32;
        position[2] += p[2] * movement_speed * delta_time as f32;

        camera.model[3][0] = position[0];
        camera.model[3][1] = position[1];
        camera.model[3][2] = position[2];

        camera.view = camera.model.inverse();

        let objects = [render::Object {
            data: &octree,
            model: Matrix::identity(),
        }];

        let batch = render::Batch {
            camera,
            objects: &objects,
        };

        vulkan.draw(batch).map_err(|e| box e)?;

        fps += 1;
    }

    //TODO figure out surface dependency on window
    //window is dropped before surface which causes segfault
    //explicit drop fixes this but it is not ideal

    drop(vulkan);
    drop(window);
    //vk shutdown happens during implicit Drop.
    //Rc ensures shutdown happens in right order.
    Ok(())
}
