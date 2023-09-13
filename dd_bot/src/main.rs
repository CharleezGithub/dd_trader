use std::process::{Command, Output};
use std::thread::sleep;
use std::time::Duration;

use enigo::*;
use rand::Rng;

#[derive(Debug)]
enum CommandError {
    ExecutionFailed(String), // This contains the error message.
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
        .arg("images/play.png")
        .output()
        .expect("Failed to execute command");

    match click_buton(&mut enigo, output, false) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Now we are opening the game
    // Run the "Ok" button detector (Will run once we enter the game)
    let output = Command::new("python")
        .arg("obj_detection.py")
        .arg("images/okay_start.png")
        .output()
        .expect("Failed to execute command");

    match click_buton(&mut enigo, output, true) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Run the "Enter the lobby" button detector
    let output = Command::new("python")
        .arg("obj_detection.py")
        .arg("images/enter_lobby.png")
        .output()
        .expect("Failed to execute command");

    match click_buton(&mut enigo, output, true) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // New the bot is in the lobby "play" tab
    // It waits untill a trade request is sent to the discord bot and then goes into the trading tab and connects to bards trade post.
    // Why bard? Because it has the least amount of active traders and therefore not as demanding to be in.
    // **After waiting..
    

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

fn click_buton(enigo: &mut Enigo, output: Output, smooth: bool) -> Result<(), CommandError> {
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

    if smooth {
        // Minize game
        enigo.key_sequence_parse("{+META}m{-META}");
        let mut rng = rand::thread_rng();
        // Randomize steps (Amount of times it moves the cursor to get to destination)
        let steps = rng.gen_range(50..100);
        // Randomize control points for bezier curve. Goes from mouse location to the end of the screen.
        let cx = rng.gen_range(enigo.mouse_location().0..enigo.main_display_size().0);
        let cy = rng.gen_range(enigo.mouse_location().1..enigo.main_display_size().1);
        // Move the cursor with the bezier function
        bezier_move(enigo, x1, y1, x2, y2, cx, cy, steps);
        // Go back into game and click the button
        enigo.key_sequence_parse("{+ALT}{+TAB}{-TAB}{-ALT}");
        enigo.mouse_click(MouseButton::Left);
        Ok(())
    } else {
        enigo.mouse_move_to(middle_point_x, middle_point_y);
        enigo.mouse_click(MouseButton::Left);
        Ok(())
    }
}

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
