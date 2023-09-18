use std::process::{Command, Output};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use enigo::*;
use rand::Rng;

// import Rocket
#[macro_use]
extern crate rocket;
use rocket::State;

#[derive(Debug)]
enum CommandError {
    ExecutionFailed(String), // This contains the error message.
}

struct TradeBotInfo {
    ready: bool,
    id: String,
}

struct Trader {
    id: String,
    discord_id: String,
}

// This function does the following:
// 1. Opens the blacksmith launcher and presses play
// 2. Goes into the lobby.
// 3. Changes the TradeBotInfo ready variable to true when ready.
async fn open_game_go_to_lobby(mut enigo: Enigo, bot_info: Arc<Mutex<TradeBotInfo>>) {
    println!("Hello from bot function!");
    tokio::time::sleep(tokio::time::Duration::from_secs(10000)).await;
    // Minimizes all tabs so that only the game is opened. To avoid clicking on other tabs
    //enigo.key_sequence_parse("{+META}m{-META}");

    // Start the launcher
    start_game(&mut enigo, "blacksmith");

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

    // Now the bot is in the lobby "play" tab
    let mut info = bot_info.lock().unwrap();
    info.ready = true;
}

// It waits untill a trade request is sent by the discord bot
fn trade(enigo: &mut Enigo, bot_info: Arc<Mutex<TradeBotInfo>>) {
    // Listen to "localhost::"
    // **After waiting..
    // Goes into the trading tab and connects to bards trade post.
    // Why bard? Because it has the least amount of active traders and therefore not as demanding to be in.
    // Run the "Trade" tab detector
    let output = Command::new("python")
        .arg("obj_detection.py")
        .arg("images/trade_tab.png")
        .output()
        .expect("Failed to execute command");

    match click_buton(enigo, output, true) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Now enter bards trading post
    // Run the "bard_trade" button detector
    let output = Command::new("python")
        .arg("obj_detection.py")
        .arg("images/bard_trade.png")
        .output()
        .expect("Failed to execute command");

    match click_buton(enigo, output, true) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Now wait for a trading request

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
        return Err(CommandError::ExecutionFailed(output_str));
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

#[get("/trade_request/<in_game_id>/<discord_id>")]
fn trade_request(
    in_game_id: String,
    discord_id: String,
    bot_info: &State<Arc<Mutex<TradeBotInfo>>>,
) -> String {
    let info = bot_info.lock().unwrap();

    if info.ready {
        format!("TradeBot ready")
    } else {
        "TradeBot not ready".into()
    }
}

fn rocket() -> rocket::Rocket<rocket::Build> {
    let enigo = Enigo::new();

    let bot_info = Arc::new(Mutex::new(TradeBotInfo {
        ready: false,
        id: "".to_string(),
    }));

    // Clone the Arc for use in main_func
    let bot_info_clone = bot_info.clone();

    // Spawn the main_func as a separate task
    tokio::spawn(async move {
        open_game_go_to_lobby(enigo, bot_info_clone).await;
    });

    rocket::build()
        .manage(bot_info) // Add the bot_info as managed state
        .mount("/", routes![trade_request])
}

#[rocket::main]
async fn main() {
    // Simply launch Rocket in the main function
    let rocket_instance = rocket();
    if let Err(err) = rocket_instance.launch().await {
        eprintln!("Rocket server error: {}", err);
    }
}
