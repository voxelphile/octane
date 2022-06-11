mod window;

use crate::window::{Event as WindowEvent, Window};

fn main() {
    println!("Hello, world!");

    let mut window = Window::new();

    window.rename("Octane");
    window.show();

    loop {
        let event = window.next_event();

        match event {
            Some(WindowEvent::KeyPress) | Some(WindowEvent::CloseRequested) => {
                break;
            }
            None => {}
        }
    }
}
