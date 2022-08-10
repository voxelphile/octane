#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
pub use win32::Window;

#[cfg(target_os = "linux")]
pub use linux::Window;

#[derive(Debug)]
pub enum Event {
    CloseRequested,
    KeyPress { keycode: Keycode },
    KeyRelease { keycode: Keycode },
    PointerMotion { x: i32, y: i32 },
    FocusIn,
    FocusOut,
    Resized { resolution: (u32, u32) },
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Keycode {
    W,
    A,
    S,
    D,
    Space,
    LeftShift,
    Escape,
}

