use std::ffi::CString;
use std::mem::MaybeUninit;

mod ffi {
    pub use libc::{c_char, c_int, c_long, c_uint, c_ulong};

    type XID = c_ulong;
    type Window = XID;
    type Atom = XID;
    type Time = c_ulong;
    type Bool = u32;

    pub const KEY_PRESS: c_int = 2;
    pub const EXPOSE: c_int = 12;
    pub const MAP_NOTIFY: c_int = 19;
    pub const REPARENT_NOTIFY: c_int = 21;
    pub const CONFIGURE_NOTIFY: c_int = 22;
    pub const CLIENT_MESSAGE: c_int = 33;

    #[derive(Clone, Copy)]
    pub enum Display {}

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ExposeEvent {
        pub ty: c_int,
        pub serial: c_ulong,
        pub send_event: Bool,
        pub display: *mut Display,
        pub window: Window,
        pub x: c_int,
        pub y: c_int,
        pub width: c_int,
        pub height: c_int,
        pub count: c_int,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct KeyEvent {
        pub ty: c_int,
        pub serial: c_ulong,
        pub send_event: Bool,
        pub display: *mut Display,
        pub window: Window,
        pub root: Window,
        pub subwindow: Window,
        pub time: Time,
        pub x: c_int,
        pub y: c_int,
        pub x_root: c_int,
        pub y_root: c_int,
        pub state: c_uint,
        pub keycode: c_uint,
        pub same_screen: Bool,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ClientMessageEvent {
        pub ty: c_int,
        pub serial: c_ulong,
        pub send_event: Bool,
        pub display: *mut Display,
        pub window: Window,
        pub message_type: Atom,
        pub format: c_int,
        pub data: [c_long; 5],
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ConfigureEvent {
        pub ty: c_int,
        pub serial: c_ulong,
        pub send_event: Bool,
        pub display: *mut Display,
        pub event: Window,
        pub window: Window,
        pub x: c_int,
        pub y: c_int,
        pub width: c_int,
        pub height: c_int,
        pub above: Window,
        pub override_redirect: Bool,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub union Event {
        pub ty: c_int,
        pub expose: ExposeEvent,
        pub key: KeyEvent,
        pub client_message: ClientMessageEvent,
        pub configure: ConfigureEvent,
        //this is a hack because event is not the right size...
        //not all implemented
        //TODO
        pub padding: [u64; 1024],
    }

    #[link(name = "X11")]
    #[allow(non_snake_case)]
    extern "C" {
        pub fn XOpenDisplay(display_name: *const c_char) -> *mut Display;
        pub fn XInitThreads() -> c_int;
        pub fn XDefaultScreen(display: *mut Display) -> c_int;
        pub fn XCreateSimpleWindow(
            display: *mut Display,
            parent: c_ulong,
            x: c_int,
            y: c_int,
            width: c_uint,
            height: c_uint,
            border_width: c_uint,
            border: c_ulong,
            background: c_ulong,
        ) -> Window;
        pub fn XRootWindow(display: *mut Display, screen_number: c_int) -> Window;
        pub fn XBlackPixel(display: *mut Display, screen_number: c_int) -> c_ulong;
        pub fn XWhitePixel(display: *mut Display, screen_number: c_int) -> c_ulong;
        pub fn XSelectInput(display: *mut Display, window: Window, event_mask: c_long) -> c_int;
        pub fn XMapWindow(display: *mut Display, window: Window);
        pub fn XUnmapWindow(display: *mut Display, window: Window);
        pub fn XNextEvent(display: *mut Display, event: *mut Event);
        pub fn XCloseDisplay(display: *mut Display);
        pub fn XInternAtom(
            display: *mut Display,
            atom_name: *const c_char,
            only_if_exists: Bool,
        ) -> Atom;
        pub fn XSetWMProtocols(
            display: *mut Display,
            window: Window,
            protocols: *const Atom,
            count: c_int,
        ) -> Bool;
        pub fn XStoreName(display: *mut Display, window: Window, window_name: *const c_char);
        pub fn XPending(display: *mut Display) -> c_int;
    }
}

pub const KEY_PRESS_MASK: i64 = 0x0000_0001;
pub const EXPOSURE_MASK: i64 = 0x0000_8000;
pub const STRUCTURE_NOTIFY_MASK: i64 = 0x0002_0000;

pub type Display = *mut ffi::Display;
pub type Screen = i32;
pub type Window = u64;
pub type Atom = u64;

pub enum Event {
    Expose {},
    KeyPress {},
    ClientMessage {
        serial: u64,
        send_event: bool,
        display: Display,
        window: Window,
        message_type: Atom,
        format: i32,
        data: [i64; 5],
    },
    ConfigureNotify {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
    ReparentNotify {},
    MapNotify {},
}

pub fn open_display(display_name: &str) -> Option<Display> {
    let display_name = CString::new(display_name).unwrap();

    let display_ptr = unsafe { ffi::XOpenDisplay(display_name.as_c_str().as_ptr()) };

    unsafe { ffi::XInitThreads() };

    if display_ptr.is_null() {
        None
    } else {
        Some(display_ptr)
    }
}

pub fn default_screen(display: Display) -> Screen {
    unsafe { ffi::XDefaultScreen(display) }
}

pub fn create_simple_window(
    display: Display,
    parent: Window,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    border_width: u32,
    border: u64,
    background: u64,
) -> Window {
    unsafe {
        ffi::XCreateSimpleWindow(
            display,
            parent,
            x,
            y,
            width,
            height,
            border_width,
            border,
            background,
        )
    }
}

pub fn root_window(display: Display, screen: Screen) -> Window {
    unsafe { ffi::XRootWindow(display, screen) }
}

pub fn black_pixel(display: Display, screen: Screen) -> u64 {
    unsafe { ffi::XBlackPixel(display, screen) }
}

pub fn white_pixel(display: Display, screen: Screen) -> u64 {
    unsafe { ffi::XWhitePixel(display, screen) }
}

pub fn select_input(display: Display, window: Window, event_mask: i64) -> i32 {
    unsafe { ffi::XSelectInput(display, window, event_mask) }
}

pub fn map_window(display: Display, window: Window) {
    unsafe { ffi::XMapWindow(display, window) };
}

pub fn unmap_window(display: Display, window: Window) {
    unsafe { ffi::XUnmapWindow(display, window) };
}

pub fn next_event(display: Display) -> Event {
    let mut event = MaybeUninit::<ffi::Event>::uninit();

    unsafe { ffi::XNextEvent(display, event.as_mut_ptr()) };

    let event = unsafe { event.assume_init() };

    unsafe {
        match event.ty {
            ffi::EXPOSE => Event::Expose {},
            ffi::KEY_PRESS => Event::KeyPress {},
            ffi::CLIENT_MESSAGE => Event::ClientMessage {
                serial: event.client_message.serial,
                send_event: event.client_message.send_event != 0,
                display: event.client_message.display,
                window: event.client_message.window,
                message_type: event.client_message.message_type,
                format: event.client_message.format,
                data: event.client_message.data,
            },
            ffi::CONFIGURE_NOTIFY => Event::ConfigureNotify {
                x: event.configure.x,
                y: event.configure.y,
                width: event.configure.width,
                height: event.configure.height,
            },
            ffi::REPARENT_NOTIFY => Event::ReparentNotify {},
            ffi::MAP_NOTIFY => Event::MapNotify {},
            _ => {
                unimplemented!("x11 event: {}", event.ty);
            }
        }
    }
}

pub fn close_display(display: Display) {
    unsafe { ffi::XCloseDisplay(display) };
}

pub fn intern_atom(display: Display, atom_name: &str, only_if_exists: bool) -> Atom {
    let atom_name = CString::new(atom_name).unwrap();

    unsafe { ffi::XInternAtom(display, atom_name.as_c_str().as_ptr(), only_if_exists as _) }
}

pub fn set_wm_protocols(display: Display, window: Window, protocols: &mut [Atom]) -> bool {
    unsafe {
        ffi::XSetWMProtocols(
            display,
            window,
            protocols.as_mut_ptr(),
            protocols.len() as i32,
        ) != 0
    }
}

pub fn store_name(display: Display, window: Window, window_name: &str) {
    let window_name = CString::new(window_name).unwrap();

    unsafe { ffi::XStoreName(display, window, window_name.as_c_str().as_ptr()) };
}

pub fn pending(display: Display) -> i32 {
    unsafe { ffi::XPending(display) }
}
