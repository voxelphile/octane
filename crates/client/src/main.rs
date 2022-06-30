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

    log::set_max_level(log::LevelFilter::Trace);
    log::set_logger(&LOGGER).expect("failed to set logger");

    let mut window = Window::new();

    window.rename("Octane");
    window.show();

    let mut vulkan = render::Vulkan::init(&window);

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

    loop {
        vulkan.draw_batch(batch.clone(), &entries);

        let event = window.next_event();

        match event {
            Some(WindowEvent::CloseRequested) => {
                break;
            }
            None => {}
            _ => {}
        }
    }

    //TODO figure out surface dependency on window
    //window is dropped before surface which causes segfault
    //explicit drop fixes this but it is not ideal
    drop(vulkan);
    drop(window);
    //vk shutdown happens during implicit Drop.
    //Rc ensures shutdown happens in right order.
}
