use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use enigo::*;
use rand::Rng;

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
        sleep(Duration::from_millis(rng.gen_range(1..3)));
        println!("{}", i)
    }
}

fn main() {
    let mut enigo = Enigo::new();

    // Minimizes all tabs so that only the game is opened. To avoid clicking on other tabs
    //enigo.key_sequence_parse("{+META}m{-META}");

    // Start the launcher
    //start_game(&mut enigo, "blacksmith");

    // Run the launcher play button detector
    let output = Command::new("python")
        .arg("obj_detection.py")
        .output()
        .expect("Failed to execute command");

    let output_str = String::from_utf8(output.stdout).unwrap();
    println!("{}", output_str);

    let (mut x1, mut y1, mut x2, mut y2) = (0, 0, 0, 0);

    if output.status.success() {
        let mut splits = output_str.trim().split_whitespace();
        x1 = splits
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or_default();
        y1 = splits
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or_default();
        x2 = splits
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or_default();
        y2 = splits
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or_default();

        println!("x1: {}, y1: {}, x2: {}, y2: {}", x1, y1, x2, y2);
    } else {
        eprintln!("Command executed with errors.\nOutput:\n{}", output_str);
    }

    // Gets the middle of the detected play button and clicks it
    let middle_point_x = ((x2 - x1) / 2) + x1;
    let middle_point_y = ((y2 - y1) / 2) + y1;

    //let mut rng = rand::thread_rng();

    enigo.mouse_move_to(middle_point_x, middle_point_y);
    enigo.mouse_click(MouseButton::Left);

    /*
    enigo.key_click(Key::Meta);

    enigo.mouse_click(MouseButton::Left)
    */
}

fn start_game(enigo: &mut Enigo, launcher_name: &str) {
    enigo.key_click(Key::Meta);
    sleep(Duration::from_millis(1000));
    enigo.key_sequence_parse(launcher_name);
    sleep(Duration::from_millis(2000));
    enigo.key_click(Key::Return);
}
