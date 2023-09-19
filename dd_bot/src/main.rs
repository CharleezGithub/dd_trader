use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use enigo::*;

// import Rocket
#[macro_use]
extern crate rocket;
use rocket::State;

mod database_functions;
mod enigo_functions;

struct TradeBotInfo {
    ready: bool,
    id: String,
}

struct Trader {
    id: String,
    discord_id: String,
    items: Vec<String>,
    // gold: i32, // IMPLEMENT THIS LATER FOR GOLD TRADES
    // has_paid_gold_fee: bool // IMPLEMENT THIS LATER FOR TRADES
}

enum TradersContainer {
    ActiveTraders(Vec<Trader>),
}

impl TradersContainer {
    fn append(&mut self, trader: Trader) {
        match self {
            TradersContainer::ActiveTraders(traders) => {
                traders.push(trader);
            }
        }
    }
}

// This function does the following:
// 1. Opens the blacksmith launcher and presses play
// 2. Goes into the lobby.
// 3. Changes the TradeBotInfo ready variable to true when ready.
async fn open_game_go_to_lobby(enigo: Arc<Mutex<Enigo>>, bot_info: Arc<Mutex<TradeBotInfo>>) {
    println!("Hello from bot function!");
    //tokio::time::sleep(tokio::time::Duration::from_secs(10000)).await;

    let mut enigo = enigo.lock().unwrap();

    // Minimizes all tabs so that only the game is opened. To avoid clicking on other tabs
    enigo.key_sequence_parse("{+META}m{-META}");

    // Start the launcher
    start_game(&mut enigo, "blacksmith");

    // Run the launcher play button detector
    let output = Command::new("python")
        .arg("obj_detection.py")
        .arg("C:/Users/Alex/Desktop/VSCode/dd_trader/dd_bot/images/play.png")
        .output()
        .expect("Failed to execute command");

    println!("Output: {:?}", output);

    match enigo_functions::click_buton(&mut enigo, output, false, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Now we are opening the game
    // Run the "Ok" button detector (Will run once we enter the game)
    let output = Command::new("python")
        .arg("obj_detection.py")
        .arg("C:/Users/Alex/Desktop/VSCode/dd_trader/dd_bot/images/okay_start.png")
        .output()
        .expect("Failed to execute command");

    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Run the "Enter the lobby" button detector
    let output = Command::new("python")
        .arg("obj_detection.py")
        .arg("C:/Users/Alex/Desktop/VSCode/dd_trader/dd_bot/images/enter_lobby.png")
        .output()
        .expect("Failed to execute command");

    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Now the bot is in the lobby "play" tab
    let mut info = bot_info.lock().unwrap();
    info.ready = true;
    info.id = String::from("Middleman2");
}

// It waits untill a trade request is sent by the discord bot
fn trade(
    enigo: &State<Arc<Mutex<Enigo>>>,
    bot_info: &State<Arc<Mutex<TradeBotInfo>>>,
    trader_id: &str,
) {
    let mut enigo = enigo.lock().unwrap();

    let info = bot_info.lock().unwrap();
    if info.ready != true {
        return;
    }
    // Goes into the trading tab and connects to bards trade post.
    // Why bard? Because it has the least amount of active traders and therefore not as demanding to be in.
    // Run the "Trade" tab detector
    let output = Command::new("python")
        .arg("obj_detection.py")
        .arg("C:/Users/Alex/Desktop/VSCode/dd_trader/dd_bot/images/trade_tab.png")
        .output()
        .expect("Failed to execute command");

    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Now enter bards trading post
    // Run the "bard_trade" button detector
    let output = Command::new("python")
        .arg("obj_detection.py")
        .arg("C:/Users/Alex/Desktop/VSCode/dd_trader/dd_bot/images/bard_trade.png")
        .output()
        .expect("Failed to execute command");

    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    //It now sends a trade to the player
    let output = Command::new("python")
        .arg("obj_detection.py")
        .arg("C:/Users/Alex/Desktop/VSCode/dd_trader/dd_bot/images/find_id.png")
        .output()
        .expect("Failed to execute command");

    // Search after the trader in the trade tab
    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    enigo.key_sequence_parse(trader_id);

    match enigo_functions::click_buton_right_direct(&mut enigo, 1824, 312, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Send a trade request
    let output = Command::new("python")
        .arg("obj_detection.py")
        .arg("C:/Users/Alex/Desktop/VSCode/dd_trader/dd_bot/images/trade_send_request.png")
        .output()
        .expect("Failed to execute command");

    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Check if trade request was accepted
    let output = Command::new("python")
        .arg("obj_detection.py")
        .arg("C:/Users/Alex/Desktop/VSCode/dd_trader/dd_bot/images/trade_send_request.png")
        .output()
        .expect("Failed to execute command");

    // Check if user has put in 50 gold for the trade fee
    let output = Command::new("python")
        .arg("obj_detection.py")
        .arg("C:/Users/Alex/Desktop/VSCode/dd_trader/dd_bot/images/gold_fee.png")
        .output()
        .expect("Failed to execute command");

    // Click the checkbox
    let output = Command::new("python")
        .arg("obj_detection.py")
        .arg("C:/Users/Alex/Desktop/VSCode/dd_trader/dd_bot/images/trade_checkbox.png")
        .output()
        .expect("Failed to execute command");

    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }
}

fn start_game(enigo: &mut Enigo, launcher_name: &str) {
    enigo.key_click(Key::Meta);
    sleep(Duration::from_millis(1000));
    enigo.key_sequence_parse(launcher_name);
    sleep(Duration::from_millis(2000));
    enigo.key_click(Key::Return);
}

#[get("/trade_request/<in_game_id>/<discord_channel_id>/<discord_id>")]
fn trade_request(
    in_game_id: &str,
    discord_channel_id: &str,
    discord_id: &str,
    enigo: &State<Arc<Mutex<Enigo>>>,
    bot_info: &State<Arc<Mutex<TradeBotInfo>>>,
    traders_container: &State<Arc<Mutex<TradersContainer>>>,
) -> String {
    let info = bot_info.lock().unwrap();
    if info.ready != true {
        return String::from("TradeBot not ready");
    }

    let mut traders = traders_container.lock().unwrap();

    // Write the database part in python first and then come back and retrive it here.
    //let trader_items =
    let item_links =
        database_functions::get_links_for_user(discord_channel_id, discord_id).unwrap();

    let trader = Trader {
        id: String::from(in_game_id),
        discord_id: String::from(discord_id),
        items: item_links,
    };

    traders.append(trader);

    trade(enigo, bot_info, in_game_id);

    format!("TradeBot ready\n{}", info.id)
}

fn rocket() -> rocket::Rocket<rocket::Build> {
    // Create 2 instances of enigo because Enigo does not implement Copy.
    let enigo = Arc::new(Mutex::new(Enigo::new()));
    let enigo2 = Arc::new(Mutex::new(Enigo::new()));

    let bot_info = Arc::new(Mutex::new(TradeBotInfo {
        ready: false,
        id: "".to_string(),
    }));

    let traders_container = Arc::new(Mutex::new(TradersContainer::ActiveTraders(Vec::new())));

    // Clone the Arc for use in main_func
    let bot_info_clone = bot_info.clone();

    // Spawn the main_func as a separate task
    tokio::spawn(async move {
        open_game_go_to_lobby(enigo2, bot_info_clone).await;
    });

    rocket::build()
        .manage(enigo) // Add the enigo as managed state
        .manage(bot_info) // Add the bot_info as managed state
        .manage(traders_container) // Add the traders_container as managed state
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
