pub enum Event {
    CloseRequested,
    KeyPress { keycode: Keycode },
    KeyRelease { keycode: Keycode },
    PointerMotion { x: i32, y: i32 },
    Resized { resolution: (u32, u32) },
}

#[derive(PartialEq, Eq, Hash)]
pub enum Keycode {
    W,
    A,
    S,
    D,
    Space,
    LeftShift,
    Escape,
}

#[cfg(target_os = "linux")]
pub use linux::Window;

#[cfg(target_os = "linux")]
mod linux {
    use super::Event;

    use std::mem;
    use std::rc::Rc;

    use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, XlibHandle};

    impl From<x11::Keycode> for super::Keycode {
        fn from(code: x11::Keycode) -> Self {
            match code {
                x11::Keycode::W => Self::W,
                x11::Keycode::A => Self::A,
                x11::Keycode::S => Self::S,
                x11::Keycode::D => Self::D,
                x11::Keycode::Space => Self::Space,
                x11::Keycode::LeftShift => Self::LeftShift,
                x11::Keycode::Escape => Self::Escape,
            }
        }
    }

    pub struct Window {
        display: x11::Display,
        window: x11::Window,
        wm_protocols: x11::Atom,
        wm_delete_window: x11::Atom,
        event_buffer: Vec<x11::Event>,
        resolution: (u32, u32),
        cursor: bool,
    }

    impl Window {
        pub fn new() -> Self {
            let resolution = (960, 540);

            let display = x11::open_display("").expect("failed to open display");

            let screen = x11::default_screen(display);

            let window = x11::create_simple_window(
                display,
                x11::root_window(display, screen),
                10,
                10,
                resolution.0,
                resolution.1,
                1,
                x11::black_pixel(display, screen),
                x11::white_pixel(display, screen),
            );

            x11::select_input(
                display,
                window,
                x11::KEY_PRESS_MASK
                    | x11::KEY_RELEASE_MASK
                    | x11::POINTER_MOTION_MASK
                    | x11::STRUCTURE_NOTIFY_MASK,
            );

            let wm_protocols = x11::intern_atom(display, "WM_PROTOCOLS", false);
            let wm_delete_window = x11::intern_atom(display, "WM_DELETE_WINDOW", false);
            let mut protocols = [wm_delete_window];

            x11::set_wm_protocols(display, window, &mut protocols);

            let event_buffer = vec![];

            let cursor = true;

            Self {
                display,
                window,
                wm_protocols,
                wm_delete_window,
                resolution,
                event_buffer,
                cursor,
            }
        }

        pub fn show(&mut self) {
            x11::map_window(self.display, self.window);
        }

        pub fn hide(&mut self) {
            x11::unmap_window(self.display, self.window);
        }

        pub fn rename(&mut self, title: &str) {
            x11::store_name(self.display, self.window, title);
        }

        pub fn show_cursor(&mut self, show: bool) {
            if show && !self.cursor {
                x11::show_cursor(self.display, self.window);
            } else if !show && self.cursor {
                x11::hide_cursor(self.display, self.window);
            }
            self.cursor = show;
            x11::flush(self.display);
        }

        pub fn capture(&mut self) {
            x11::warp_pointer(
                self.display,
                self.window,
                0,
                0,
                0,
                0,
                self.resolution.0 as i32 / 2,
                self.resolution.1 as i32 / 2,
            );
        }

        pub fn next_event(&mut self) -> Option<Event> {
            while x11::pending(self.display) > 0 {
                let event = x11::next_event(self.display);

                if let Ok(event) = event {
                    self.event_buffer.push(event);
                }
            }

            //remove autorepeats
            let mut i = 1;
            while i < self.event_buffer.len() {
                if let x11::Event::KeyPress {
                    keycode: key_press_code,
                    serial: key_press_serial,
                } = &self.event_buffer[i]
                {
                    if let x11::Event::KeyRelease {
                        keycode: key_release_code,
                        serial: key_release_serial,
                    } = &self.event_buffer[i - 1]
                    {
                        if key_press_code == key_release_code
                            && key_press_serial == key_release_serial
                        {
                            self.event_buffer.remove(i);
                            self.event_buffer.remove(i - 1);
                        }
                    }
                }

                i += 1;
            }

            if self.event_buffer.len() == 0 {
                return None;
            }

            match self.event_buffer.remove(0) {
                x11::Event::KeyPress { keycode, .. } => Some(Event::KeyPress {
                    keycode: keycode.into(),
                }),
                x11::Event::KeyRelease { keycode, .. } => Some(Event::KeyRelease {
                    keycode: keycode.into(),
                }),
                x11::Event::MotionNotify { x, y } => Some(Event::PointerMotion { x, y }),
                x11::Event::ClientMessage {
                    message_type,
                    format,
                    data,
                    ..
                } => {
                    if message_type == self.wm_protocols && format == 32 {
                        let protocol = data[0] as x11::Atom;

                        if protocol == self.wm_delete_window {
                            return Some(Event::CloseRequested);
                        }
                    }

                    None
                }
                x11::Event::ConfigureNotify { width, height, .. } => {
                    let width = width as _;
                    let height = height as _;

                    if self.resolution != (width, height) {
                        self.resolution = (width, height);

                        Some(Event::Resized {
                            resolution: self.resolution,
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }

        pub fn resolution(&self) -> (u32, u32) {
            self.resolution
        }
    }

    impl Drop for Window {
        fn drop(&mut self) {
            //This is a crazy hack to prevent segfault
            //x11::close_display(self.display);
        }
    }

    unsafe impl HasRawWindowHandle for Window {
        fn raw_window_handle(&self) -> RawWindowHandle {
            //xlib handle is non exhaustive
            let mut xlib_handle = XlibHandle::empty();

            xlib_handle.window = self.window;
            xlib_handle.display = unsafe { mem::transmute(self.display) };

            RawWindowHandle::Xlib(xlib_handle)
        }
    }
}
