use super::{Event, Keycode};

use std::ptr;
use std::sync::{Arc, Mutex};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, Win32Handle};

impl From<windows::Keycode> for Keycode {
    fn from(keycode: windows::Keycode) -> Self {
        match keycode {
            windows::Keycode::W => Self::W,
            windows::Keycode::A => Self::A,
            windows::Keycode::S => Self::S,
            windows::Keycode::D => Self::D,
            windows::Keycode::Space => Self::Space,
            windows::Keycode::LeftShift => Self::LeftShift,
            windows::Keycode::Escape => Self::Escape,
        }
    }
}

pub struct Window {
    hwnd: windows::Hwnd,
    fullscreen: bool,
    placement: windows::WindowPlacement,
    style: windows::WindowStyle,
    style_ex: windows::WindowStyleEx,
    queue: windows::Queue,
}

impl Window {
    pub fn new() -> Self {

        let wnd_class = windows::WndClass {
            class_name: "Octane",
        };

        windows::register_class(wnd_class);

        let queue = Box::leak(box Mutex::new(vec![]));

        let hwnd = windows::create_window(windows::WindowCreateInfo {
            class_name: "Octane", 
            window_name: "Octane", 
            x: None,
            y: None,
            width: Some(960),
            height: Some(540),
            parent: None,
            queue,
        });

        let Some(hwnd) = hwnd else { panic!("hwnd is zero") };

        let fullscreen = false;

        let placement = windows::get_window_placement(hwnd);

        let style = windows::get_window_style(hwnd);

        let style_ex = windows::get_window_style_ex(hwnd);

        Self { hwnd, fullscreen, placement, style, style_ex, queue }
    }

    pub fn show(&mut self) {
        windows::show_window(self.hwnd, windows::ShowCmd::SHOW);
    }

    pub fn hide(&mut self) {
        windows::show_window(self.hwnd, windows::ShowCmd::HIDE);
    }

    pub fn rename(&mut self, title: &str) {
        windows::set_window_text(self.hwnd, title);
    }

    pub fn fullscreen(&mut self, fullscreen: bool) {
        if self.fullscreen == fullscreen {
            return;
        }

        if self.fullscreen {
            windows::set_window_style(self.hwnd, self.style);
            windows::set_window_style_ex(self.hwnd, self.style_ex);
            windows::show_window(self.hwnd, windows::ShowCmd::SHOW_NORMAL);
            windows::set_window_placement(self.hwnd, self.placement);
        } else {
            self.placement = windows::get_window_placement(self.hwnd);

            let mut new_style = self.style;
            new_style &= !windows::WindowStyle::BORDER; 
            new_style &= !windows::WindowStyle::DLG_FRAME;
            new_style &= !windows::WindowStyle::THICK_FRAME;

            let mut new_style_ex = self.style_ex;
            new_style_ex &= !windows::WindowStyleEx::WINDOW_EDGE;

            windows::set_window_style(self.hwnd, new_style);
            windows::set_window_style_ex(self.hwnd, new_style_ex);
            windows::show_window(self.hwnd, windows::ShowCmd::SHOW_MAXIMIZED);

        }

        self.fullscreen = fullscreen;
    }

    pub fn show_cursor(&mut self, show: bool) {
        windows::show_cursor(show);
    }

    pub fn capture(&mut self) {
        let (x, y) = self.center();
        windows::set_cursor_pos(x as _, y as _);
    }

    pub fn next_event(&mut self) -> Option<Event> {
        while let Some(msg) = windows::peek_message(self.hwnd, true) {
            windows::translate_message(&msg);
            windows::dispatch_message(&msg);
        }
        
        let mut queue_guard = unsafe { self.queue.as_mut().unwrap() }.lock().unwrap();

        if let Some(&event) = queue_guard.get(0) {
            queue_guard.remove(0);
            Some(match event {
                windows::Event::CloseRequested => 
                    Event::CloseRequested,
                windows::Event::KeyPress { keycode } => 
                    Event::KeyPress {
                        keycode: keycode.into(),
                    },
                windows::Event::KeyRelease { keycode } =>
                    Event::KeyRelease {
                        keycode: keycode.into(),
                    },
                windows::Event::PointerMotion { x, mut y } => {
                    y += if self.fullscreen { 0 } else { 11 };  
                    Event::PointerMotion { x, y }
                }
                windows::Event::FocusIn =>
                    Event::FocusIn,
                windows::Event::FocusOut =>
                    Event::FocusOut,
                windows::Event::Resized { resolution } =>
                    Event::Resized { resolution },
            })
        } else {
            None
        }
    }

    pub fn resolution(&self) -> (u32, u32) {
        let windows::Rect(left, top, right, bottom) = windows::get_client_rect(self.hwnd);

        (right as _, bottom as _)
    }

    pub fn center(&self) -> (u32, u32) {
        let windows::Rect(left, top, right, bottom) = windows::get_window_rect(self.hwnd);
        ((left + (right - left) / 2) as u32, (top + (bottom - top)  / 2) as u32)
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut win32_handle = Win32Handle::empty();

        win32_handle.hwnd = self.hwnd as _;

        RawWindowHandle::Win32(win32_handle)
    }
}
