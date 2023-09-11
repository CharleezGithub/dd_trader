use enigo::{Enigo, MouseControllable};
use std::thread::sleep;
use std::time::Duration;

fn smooth_move(enigo: &mut Enigo, x1: i32, y1: i32, x2: i32, y2: i32, steps: usize) {
    let dx = x2 - x1;
    let dy = y2 - y1;

    for i in 1..=steps {
        let x = x1 + (dx * i as i32) / steps as i32;
        let y = y1 + (dy * i as i32) / steps as i32;
        enigo.mouse_move_to(x, y);
        sleep(Duration::from_millis(10)); // Adjust this delay as needed
    }
}

fn main() {
    let mut enigo = Enigo::new();

    let start_x = 100;
    let start_y = 100;
    let end_x = 500;
    let end_y = 500;

    smooth_move(&mut enigo, start_x, start_y, end_x, end_y, 100);
}
