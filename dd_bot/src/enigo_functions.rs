use std::process::Output;
use std::thread::sleep;
use std::time::Duration;

use enigo::*;
use rand::Rng;

#[derive(Debug)]
pub enum CommandError {
    ExecutionFailed(String), // This contains the error message.
}

pub fn click_buton(
    enigo: &mut Enigo,
    output: Output,
    smooth: bool,
    offset_x: i32,
    offset_y: i32,
) -> Result<(), CommandError> {
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
        return Err(CommandError::ExecutionFailed(output_str));
    }

    let mut rng = rand::thread_rng();

    // Salt the pixels so that it does not click the same pixel every time.
    let salt = rng.gen_range(1..9);

    // Gets the middle of the detected play button and clicks it
    let middle_point_x = ((x2 - x1) / 2) + x1 + offset_x + salt;
    let middle_point_y = ((y2 - y1) / 2) + y1 + offset_y + salt;

    if smooth {
        // Minize game
        enigo.key_sequence_parse("{+META}m{-META}");
        // Randomize steps (Amount of times it moves the cursor to get to destination)
        let steps = rng.gen_range(50..100);
        // Randomize control points for bezier curve. Goes from mouse location to the end of the screen.
        let cx = rng.gen_range(enigo.mouse_location().0..enigo.main_display_size().0);
        let cy = rng.gen_range(enigo.mouse_location().1..enigo.main_display_size().1);
        // Move the cursor with the bezier function
        bezier_move(enigo, x1, y1, middle_point_x, middle_point_y, cx, cy, steps);
        // Go back into game and click the button
        enigo.key_sequence_parse("{+ALT}{+TAB}");
        sleep(Duration::from_millis(rng.gen_range(50..70)));
        enigo.key_sequence_parse("{-TAB}{-ALT}");
        sleep(Duration::from_millis(rng.gen_range(500..1000)));
        enigo.mouse_click(MouseButton::Left);
        Ok(())
    } else {
        enigo.mouse_move_to(middle_point_x, middle_point_y);
        enigo.mouse_click(MouseButton::Left);
        Ok(())
    }
}

pub fn click_buton_right(
    enigo: &mut Enigo,
    output: Output,
    smooth: bool,
    offset_x: i32,
    offset_y: i32,
) -> Result<(), CommandError> {
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
        return Err(CommandError::ExecutionFailed(output_str));
    }

    // Gets the middle of the detected play button and clicks it
    let middle_point_x = ((x2 - x1) / 2) + x1 + offset_x;
    let middle_point_y = ((y2 - y1) / 2) + y1 + offset_y;

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
        bezier_move(enigo, x1, y1, middle_point_x, middle_point_y, cx, cy, steps);
        // Go back into game and click the button
        enigo.key_sequence_parse("{+ALT}{+TAB}{-TAB}{-ALT}");
        enigo.mouse_click(MouseButton::Right);
        Ok(())
    } else {
        enigo.mouse_move_to(middle_point_x, middle_point_y);
        enigo.mouse_click(MouseButton::Right);
        Ok(())
    }
}

pub fn click_buton_direct(
    enigo: &mut Enigo,
    x: i32,
    y: i32,
    smooth: bool,
    offset_x: i32,
    offset_y: i32,
) -> Result<(), CommandError> {
    println!("{}, {}", x, y);

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
        bezier_move(
            enigo,
            enigo.mouse_location().0,
            enigo.mouse_location().1,
            x + offset_x,
            y + offset_y,
            cx,
            cy,
            steps,
        );
        // Go back into game and click the button
        enigo.key_sequence_parse("{+ALT}{+TAB}{-TAB}{-ALT}");
        enigo.mouse_click(MouseButton::Left);
        Ok(())
    } else {
        enigo.mouse_move_to(x, y);
        enigo.mouse_click(MouseButton::Left);
        Ok(())
    }
}
pub fn click_buton_right_direct(
    enigo: &mut Enigo,
    x: i32,
    y: i32,
    smooth: bool,
    offset_x: i32,
    offset_y: i32,
) -> Result<(), CommandError> {
    println!("{}, {}", x, y);

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
        bezier_move(
            enigo,
            enigo.mouse_location().0,
            enigo.mouse_location().1,
            x + offset_x,
            y + offset_y,
            cx,
            cy,
            steps,
        );
        // Go back into game and click the button
        enigo.key_sequence_parse("{+ALT}{+TAB}{-TAB}{-ALT}");
        enigo.mouse_click(MouseButton::Right);
        Ok(())
    } else {
        enigo.mouse_move_to(x, y);
        enigo.mouse_click(MouseButton::Right);
        Ok(())
    }
}

pub fn bezier_move(
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
