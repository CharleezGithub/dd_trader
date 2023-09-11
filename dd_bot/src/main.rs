use enigo::{Enigo, MouseControllable};
use rand::Rng;
use std::thread::sleep;
use std::time::Duration;

fn bezier_move(
    enigo: &mut Enigo,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    cx: i32,
    cy: i32,
    steps: usize,
) {
    let mut rng = rand::thread_rng();
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let x =
            (1.0 - t).powi(2) * x1 as f32 + 2.0 * (1.0 - t) * t * cx as f32 + t.powi(2) * x2 as f32;
        let y =
            (1.0 - t).powi(2) * y1 as f32 + 2.0 * (1.0 - t) * t * cy as f32 + t.powi(2) * y2 as f32;
        enigo.mouse_move_to(x.round() as i32, y.round() as i32);
        sleep(Duration::from_millis(10)); // Does not work as intended.
        println!("{}", i)
    }
}

fn main() {
    let mut enigo = Enigo::new();

    let mut rng = rand::thread_rng();

    let steps = rng.gen_range(700..701);

    let screen_size = enigo.main_display_size();

    // If 0 then the control point will be (960, 0) (upper) else (960, full height of screen)
    let upper_or_under = rng.gen_range(0..1);

    let start_x = 100;
    let start_y = 100;
    let control_x = screen_size.0 / 2; // Control point for the Bézier curve
    let control_y: i32; // Control point for the Bézier curve

    // upper
    if upper_or_under == 0 {
        control_y = 0;
    }
    // lower
    else {
        control_y = screen_size.1;
    }

    let end_x = 1900;
    let end_y = 1000;

    println!(
        "Start: {:?}\nEnd: {:?}\nControl: {:?}",
        (start_x, start_y),
        (end_x, end_y),
        (control_x, control_y)
    );

    bezier_move(
        &mut enigo, start_x, start_y, end_x, end_y, control_x, control_y, steps,
    );
}
