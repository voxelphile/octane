use super::Event;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, XlibHandle};

pub struct Window {
}

impl Window {
    pub fn new() -> Self {
        todo!()
    }

    pub fn show(&mut self) {
        todo!()
    }

    pub fn hide(&mut self) {
        todo!()
    }

    pub fn rename(&mut self, title: &str) {
        todo!()
    }

    pub fn fullscreen(&mut self, fullscreen: bool) {
        todo!()
    }

    pub fn show_cursor(&mut self, show: bool) {
        todo!()
    }

    pub fn capture(&mut self) {
        todo!()
    }

    pub fn next_event(&mut self) -> Option<Event> {
        todo!()
    }

    pub fn resolution(&self) -> (u32, u32) {
        todo!()
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        todo!()
    }
}
