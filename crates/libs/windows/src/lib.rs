#![allow(dead_code)]
#![feature(box_syntax)]
#![feature(try_blocks)]
#![feature(ptr_to_from_bits)]

use std::ptr;
use std::mem;
use std::iter;
use std::sync::{Arc, Mutex};

use bitflags::bitflags;

pub mod ffi;

const CREATE: u32 = 0x1;
const CLOSE: u32 = 0x0010;
const MOUSE_MOVE: u32 = 0x0200;
const KEY_DOWN: u32 = 0x0100;
const KEY_UP: u32 = 0x0101;
const KILL_FOCUS: u32 = 0x0008;
const SET_FOCUS: u32 = 0x0007;
const SIZE: u32 = 0x0005;

#[derive(Debug, Clone, Copy)]
pub enum Keycode {
    W,
    A,
    S,
    D,
    Space,
    LeftShift,
    Escape,
}

impl TryFrom<usize> for Keycode {
    type Error = ();
    fn try_from(bits: usize) -> Result<Keycode, Self::Error> {
        Ok(match bits {
            0x57 => Keycode::W,
            0x41 => Keycode::A,
            0x53 => Keycode::S,
            0x44 => Keycode::D,
            0x20 => Keycode::Space,
            0x10 => Keycode::LeftShift,
            0x1B => Keycode::Escape,
            _ => Err(())?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Event {
    CloseRequested,
    KeyPress { keycode: Keycode },
    KeyRelease { keycode: Keycode },
    PointerMotion { x: i32, y: i32 },
    FocusIn,
    FocusOut,
    Resized { resolution: (u32, u32) },
}

pub extern "system" fn wnd_proc(hwnd: ffi::Hwnd, u_msg: u32, w_param: ffi::WParam, l_param: ffi::LParam) -> ffi::LResult {
    let event = try { match u_msg {
        CREATE => {
            unsafe { ffi::SetWindowLongPtrA(hwnd, Gwl::USER_DATA.bits(), *l_param as ffi::LongPtr) };
            None?
        },
        CLOSE => {
            Event::CloseRequested
        },
        KEY_DOWN => {
            Event::KeyPress {
                keycode: Keycode::try_from(w_param as usize).ok()?
            }
        },
        KEY_UP => {
            Event::KeyRelease {
                keycode: Keycode::try_from(w_param as usize).ok()?
            }
        },
        MOUSE_MOVE => {
            Event::PointerMotion {
                x: (((l_param as usize) >> 0) 
                    & 0xFFFF) as i32,
                y: (((l_param as usize) >> 16) 
                    & 0xFFFF) as i32
            }
        },
        SET_FOCUS => {
            Event::FocusIn
        },
        KILL_FOCUS => {
            Event::FocusOut
        },
        SIZE => {
            Event::Resized {
                resolution: (
                    (((l_param as usize) >> 0) 
                        & 0xFFFF) as u32,
                    (((l_param as usize) >> 16) 
                        & 0xFFFF) as u32
                ),
            }
        },
        _ => None?
    } }; 

    if let Some(event) = event {
        let queue = unsafe { ffi::GetWindowLongPtrA(hwnd, Gwl::USER_DATA.bits()) } as Queue;
        let queue = unsafe { queue.as_mut().unwrap() };
        let mut queue = queue.lock().unwrap();
        queue.push(event);
        ptr::null_mut()
    } else {
        unsafe { ffi::DefWindowProcA(hwnd, u_msg, w_param, l_param) }
    }
}

pub type Hwnd = usize;

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Point(pub i32, pub i32);
#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Rect(pub i32, pub i32, pub i32, pub i32);

pub type WParam = ffi::WParam;
pub type LParam = ffi::LParam;
pub type LResult = ffi::LResult;

pub type Queue = *mut Mutex<Vec<Event>>;

pub struct WndClass<'a> {
    pub class_name: &'a str,
}

pub fn register_class(wnd_class: WndClass<'_>) {
    let class_name = std::ffi::CString::new(wnd_class.class_name).unwrap();

    let wnd_class_a = ffi::WndClassA {
        style: 0,
        wnd_proc,
        cls_extra: 0,
        wnd_extra: 0,
        instance: ptr::null_mut(),
        icon: ptr::null_mut(),
        cursor: ptr::null_mut(),
        background: ptr::null_mut(),
        menu_name: ptr::null_mut(),
        class_name: class_name.as_ptr() as _,
    };

    unsafe { ffi::RegisterClassA(&wnd_class_a) };
}

pub struct WindowCreateInfo<'a> {
    pub class_name: &'a str,
    pub window_name: &'a str,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub parent: Option<Hwnd>,
    pub queue: Queue,
}

pub fn create_window(info: WindowCreateInfo<'_>) -> Option<Hwnd> {
    let class_name = std::ffi::CString::new(info.class_name).unwrap();
    let window_name = std::ffi::CString::new(info.window_name).unwrap();

    let hwnd = unsafe { ffi::CreateWindowExA(
        0, 
        class_name.as_ptr() as _, 
        window_name.as_ptr() as _, 
        WindowStyle::OVERLAPPED_WINDOW.bits() as _,
        info.x.unwrap_or(ffi::USE_DEFAULT),
        info.y.unwrap_or(ffi::USE_DEFAULT),
        info.width.unwrap_or(ffi::USE_DEFAULT),
        info.height.unwrap_or(ffi::USE_DEFAULT),
        info.parent.map_or(ptr::null_mut(), |x| x as _),
        ptr::null_mut(),
        ptr::null_mut(),
        info.queue as _,
    )};

    if hwnd.is_null() {
        None
    } else {
        Some(hwnd as _)
    }
}

bitflags! {
    #[repr(transparent)]
    pub struct ShowCmd: i32 {
        const HIDE = 0;
        const SHOW_NORMAL = 1;
        const SHOW_MAXIMIZED = 3;
        const SHOW = 5;
    }
}

bitflags! {
    #[repr(transparent)]
    pub struct WindowStyle: i32 {
        const OVERLAPPED = 0x00000000;
        const MAXIMIZE_BOX = 0x00010000;
        const MINIMIZE_BOX = 0x00020000;
        const SYS_MENU = 0x80000;
        const CAPTION = 0xc00000;
        const SIZE_FRAME = 0x40000;
        const BORDER = 0x00800000;
        const DLG_FRAME = 0x00400000;
        const THICK_FRAME = 0x00040000; 
        const OVERLAPPED_WINDOW = 
            Self::OVERLAPPED.bits 
            | Self::CAPTION.bits 
            | Self::SYS_MENU.bits 
            | Self::THICK_FRAME.bits 
            | Self::MINIMIZE_BOX.bits
            | Self::MAXIMIZE_BOX.bits;
    }
}

bitflags! {
    #[repr(transparent)]
    pub struct WindowStyleEx: i32 {
        const WINDOW_EDGE = 0x00000100;
    }
}

pub fn show_window(hwnd: Hwnd, cmd: ShowCmd) {
    unsafe { ffi::ShowWindow(hwnd as _, cmd.bits()) };
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct WindowPlacement {
    pub flags: u32,
    pub show_cmd: u32,
    pub min_position: Point,
    pub max_position: Point,
    pub normal_position: Rect,
    pub device: Rect,
}

pub fn get_window_placement(hwnd: Hwnd) -> WindowPlacement {
    let mut wp = ffi::WindowPlacement {
        length: mem::size_of::<ffi::WindowPlacement>() as _,
        flags: 0,
        show_cmd: 0,
        min_position: Default::default(),
        max_position: Default::default(),
        normal_position: Default::default(),
        device: Default::default(),
    };

    unsafe { ffi::GetWindowPlacement(hwnd as _, &mut wp) };

    WindowPlacement {
        flags: wp.flags,
        show_cmd: wp.show_cmd,
        min_position: unsafe { mem::transmute(wp.min_position) },
        max_position: unsafe { mem::transmute(wp.max_position) },
        normal_position: unsafe { mem::transmute(wp.normal_position) },
        device: unsafe { mem::transmute(wp.device) },
    }
}

pub fn set_window_placement(hwnd: Hwnd, wp: WindowPlacement) {
    let wp = ffi::WindowPlacement {
        length: mem::size_of::<ffi::WindowPlacement>() as _,
        flags: wp.flags,
        show_cmd: wp.show_cmd,
        min_position: unsafe { mem::transmute(wp.min_position) },
        max_position: unsafe { mem::transmute(wp.max_position) },
        normal_position: unsafe { mem::transmute(wp.normal_position) },
        device: unsafe { mem::transmute(wp.device) },
    };

    unsafe { ffi::SetWindowPlacement(hwnd as _, &wp) };
}

bitflags! {
    pub struct Gwl: i32 {
        const STYLE = -16;
        const EX_STYLE = -20;
        const USER_DATA = -21;
    }
}

pub fn get_window_style(hwnd: Hwnd) -> WindowStyle {
    let bits = unsafe { ffi::GetWindowLongA(hwnd as _, Gwl::STYLE.bits()) };

    unsafe { mem::transmute(bits) }
}

pub fn set_window_style(hwnd: Hwnd, new: WindowStyle) {
    unsafe { ffi::SetWindowLongA(hwnd as _, Gwl::STYLE.bits(), new.bits()) };
}

pub fn get_window_style_ex(hwnd: Hwnd) -> WindowStyleEx {
    let bits = unsafe { ffi::GetWindowLongA(hwnd as _, Gwl::EX_STYLE.bits()) };

    unsafe { mem::transmute(bits) }
}

pub fn set_window_style_ex(hwnd: Hwnd, new: WindowStyleEx) {
    unsafe { ffi::SetWindowLongA(hwnd as _, Gwl::EX_STYLE.bits(), new.bits()) };
}

pub fn set_window_text(hwnd: Hwnd, text: &'_ str) {
    let text = std::ffi::CString::new(text).unwrap();

    unsafe { ffi::SetWindowTextA(hwnd as _, text.as_ptr() as _) };
}

pub fn show_cursor(show: bool) {
    unsafe { ffi::ShowCursor(show as _) };
}

pub fn get_window_rect(hwnd: Hwnd) -> Rect {
    let mut rect = unsafe { mem::zeroed::<ffi::Rect>() };

    unsafe { ffi::GetWindowRect(hwnd as _, &mut rect) };

    unsafe { mem::transmute(rect) }
}

pub fn get_client_rect(hwnd: Hwnd) -> Rect {
    let mut rect = unsafe { mem::zeroed::<ffi::Rect>() };

    unsafe { ffi::GetClientRect(hwnd as _, &mut rect) };

    unsafe { mem::transmute(rect) }
}

pub fn set_cursor_pos(x: i32, y: i32) {
    unsafe { ffi::SetCursorPos(x, y) };
}  

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Msg {
    hwnd: Hwnd,
    message: u32,
    w_param: WParam,
    l_param: LParam,
    time: u32,
    pt: Point,
    l_private: u32,
}

pub fn peek_message(hwnd: Hwnd, remove: bool) -> Option<Msg> {
    let mut msg = unsafe { mem::zeroed::<ffi::Msg>() };

    let result = unsafe { ffi::PeekMessageA(&mut msg, hwnd as _, 0, 0, remove as _) };

    if result > 0 {
        Some(unsafe { mem::transmute(msg) })
    } else { 
        None
    }
}

pub fn translate_message(msg: &'_ Msg) {
    unsafe { ffi::TranslateMessage(msg as *const _ as _) };
}

pub fn dispatch_message(msg: &'_ Msg) {
    unsafe { ffi::DispatchMessageW(msg as *const _ as _) };
}

pub fn set_capture(hwnd: Hwnd) {
    unsafe { ffi::SetCapture(hwnd as _) };
}

pub fn release_capture() {
    unsafe { ffi::ReleaseCapture() };
}
