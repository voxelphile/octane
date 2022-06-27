pub enum Event {
    CloseRequested,
    KeyPress,
}

#[cfg(target_os = "linux")]
pub use linux::Window;

#[cfg(target_os = "linux")]
mod linux {
    use super::Event;

    use std::mem;
    use std::rc::Rc;

    use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, XlibHandle};

    pub struct Window {
        display: x11::Display,
        window: x11::Window,
        wm_protocols: x11::Atom,
        wm_delete_window: x11::Atom,
    }

    impl Window {
        pub fn new() -> Self {
            let display = x11::open_display("").expect("failed to open display");

            let screen = x11::default_screen(display);

            let window = x11::create_simple_window(
                display,
                x11::root_window(display, screen),
                10,
                10,
                2560,
                1440,
                1,
                x11::black_pixel(display, screen),
                x11::white_pixel(display, screen),
            );

            x11::select_input(display, window, x11::KEY_PRESS_MASK);

            let wm_protocols = x11::intern_atom(display, "WM_PROTOCOLS", false);
            let wm_delete_window = x11::intern_atom(display, "WM_DELETE_WINDOW", false);
            let mut protocols = [wm_delete_window];

            x11::set_wm_protocols(display, window, &mut protocols);

            Self {
                display,
                window,
                wm_protocols,
                wm_delete_window,
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

        pub fn next_event(&mut self) -> Option<Event> {
            if x11::pending(self.display) == 0 {
                return None;
            }

            let event = x11::next_event(self.display);

            match event {
                x11::Event::KeyPress {} => Some(Event::KeyPress),
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
                _ => None,
            }
        }
    }

    impl Drop for Window {
        fn drop(&mut self) {
            x11::close_display(self.display);
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
