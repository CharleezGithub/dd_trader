use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::{result, str};
use std::process::{Command, exit};

use enigo::*;

// import Rocket
#[macro_use]
extern crate rocket;
use rocket::State;

use rocket::futures::stream::iter;
use rocket::response::stream::EventStream;

use rocket::response::stream::Event;
use rocket::time::Duration;
use trading_functions::collect_gold_fee;

mod database_functions;
mod enigo_functions;
mod trading_functions;

use std::fs::{read_to_string, write, OpenOptions};
use std::io::{self, Seek, SeekFrom};

pub enum ReadyState {
    False,
    True,
    Starting,
}

pub struct TradeBotInfo {
    ready: ReadyState,
    id: String,
}

#[derive(Clone)]
pub struct Trader {
    in_game_id: String,
    discord_channel_id: String,
    discord_id: String,
    item_images: Vec<String>,
    info_images: Vec<String>,
    gold: i32,
    has_paid_gold_fee: bool,
}

pub enum TradersContainer {
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

    fn get_trader_by_in_game_id(&self, in_game_id: &str) -> Option<&Trader> {
        match self {
            TradersContainer::ActiveTraders(traders) => traders
                .iter()
                .find(|trader| trader.in_game_id.to_lowercase() == in_game_id.to_lowercase()),
        }
    }

    fn get_other_trader_in_channel(
        &self,
        discord_id: &str,
        discord_channel_id: &str,
    ) -> Option<&Trader> {
        match self {
            TradersContainer::ActiveTraders(traders) => {
                for trader in traders.iter() {
                    if trader.discord_channel_id == discord_channel_id
                        && trader.discord_id != discord_id
                    {
                        return Some(trader);
                    }
                }
            }
        }
        None
    }

    fn update_gold_fee_status(&mut self, in_game_id: &str, new_status: bool) {
        match self {
            TradersContainer::ActiveTraders(traders) => {
                if let Some(trader) = traders
                    .iter_mut()
                    .find(|trader| trader.in_game_id == in_game_id)
                {
                    trader.has_paid_gold_fee = new_status;
                }
            }
        }
    }

    fn set_in_game_id_by_discord_info(
        &mut self,
        new_in_game_id: &str,
        discord_id: &str,
        discord_channel_id: &str,
    ) {
        match self {
            TradersContainer::ActiveTraders(traders) => {
                for trader in traders.iter_mut() {
                    if trader.discord_id == discord_id
                        && trader.discord_channel_id == discord_channel_id
                    {
                        trader.in_game_id = new_in_game_id.to_string();
                        println!(
                            "Updated in_game_id for discord_id {}: {}",
                            discord_id, new_in_game_id
                        );
                        break;
                    }
                }
            }
        }
    }
}

#[get("/restart")]
fn restart() {
    use std::env;

    let exe_path = env::current_exe().unwrap();

    Command::new(exe_path)
        .spawn()
        .expect("Failed to restart the program");

    exit(0);
}

#[get("/gold_fee/<in_game_id>/<discord_channel_id>/<discord_id>")]
fn gold_fee(
    in_game_id: String,
    discord_channel_id: &str,
    discord_id: &str,
    enigo: &State<Arc<Mutex<Enigo>>>,
    bot_info: &State<Arc<Mutex<TradeBotInfo>>>,
    traders_container: &State<Arc<Mutex<TradersContainer>>>,
) -> String {
    {
        let mut traders = traders_container.lock().unwrap();
        traders.set_in_game_id_by_discord_info(in_game_id.as_str(), discord_id, discord_channel_id);
    }

    // Dereference `State` and clone the inner `Arc`.
    let enigo_cloned = enigo.inner().clone();
    let bot_info_cloned = bot_info.inner().clone();
    let traders_container_cloned = traders_container.inner().clone();
    let in_game_id_clone = in_game_id.clone();

    // Spawn the trading function in a non-blocking way
    tokio::spawn(async move {
        // Any code that runs here is non-blocking to the main thread
        let result = tokio::task::spawn_blocking(move || {
            trading_functions::collect_gold_fee(
                (&enigo_cloned).into(),
                (&bot_info_cloned).into(),
                &in_game_id_clone,
                (&traders_container_cloned).into(),
            )
        })
        .await
        .unwrap_or_else(|e| Err(e.to_string())); // Handle errors

        tokio::task::yield_now().await;

        // Log the result or handle it further, based on requirements
        match result {
            Ok(s) => {
                let _ = update_status(s.as_str());
                println!("Trade result: {}", s);
            }
            Err(e) => {
                let _ = update_status(format!("{:?}", e).as_str());
                eprintln!("Trade error: {:?}", e)
            }
        }
    });

    let info = bot_info.lock().unwrap();
    let _ = match info.ready {
        ReadyState::False => {
            let _ = update_status("TradeBot not ready");
            return String::from("TradeBot not ready");
        }
        ReadyState::Starting => {
            let _ = update_status("TradeBot is starting. Please wait 2 minutes.");
            return String::from("TradeBot is starting. Please wait 2 minutes.");
        }
        ReadyState::True => {
            let _ = update_status("TradeBot ready");
            return format!(
                "Sending `{}` a trade request in `The Bard's Theater #1` trading channel",
                in_game_id
            );
        }
    };
}

#[get("/deposit/<in_game_id>/<discord_channel_id>/<discord_id>")]
fn deposit(
    in_game_id: String,
    discord_channel_id: String,
    discord_id: String,
    enigo: &State<Arc<Mutex<Enigo>>>,
    bot_info: &State<Arc<Mutex<TradeBotInfo>>>,
    traders_container: &State<Arc<Mutex<TradersContainer>>>,
) -> String {
    {
        let mut traders = traders_container.lock().unwrap();
        traders.set_in_game_id_by_discord_info(&in_game_id, &discord_id, &discord_channel_id);
    }

    // Check the bot status before proceeding
    let info = bot_info.lock().unwrap();
    match info.ready {
        ReadyState::False => {
            update_status("TradeBot not ready").unwrap();
            return String::from("TradeBot not ready");
        }
        ReadyState::Starting => {
            update_status("TradeBot is starting. Please wait 2 minutes.").unwrap();
            return String::from("TradeBot is starting. Please wait 2 minutes.");
        }
        ReadyState::True => {
            update_status("TradeBot ready").unwrap();
        }
    }

    // Dereference `State` and clone the inner `Arc`.
    let enigo_cloned = enigo.inner().clone();
    let bot_info_cloned = bot_info.inner().clone();
    let traders_container_cloned = traders_container.inner().clone();
    let in_game_id_cloned = in_game_id.clone();

    // Spawn the trading function in a non-blocking way
    tokio::spawn(async move {
        // Any code that runs here is non-blocking to the main thread
        let result = tokio::task::spawn_blocking(move || {
            trading_functions::deposit(
                (&enigo_cloned).into(),
                (&bot_info_cloned).into(),
                &in_game_id_cloned,
                (&traders_container_cloned).into(),
            )
        })
        .await
        .unwrap_or_else(|e| Err(e.to_string())); // Handle errors

        tokio::task::yield_now().await;

        // Log the result or handle it further, based on requirements
        match result {
            Ok(s) => {
                let _ = update_status(s.as_str());
                println!("Trade result: {}", s);
            }
            Err(e) => {
                let _ = update_status(format!("{:?}", e).as_str());
                eprintln!("Trade error: {:?}", e)
            }
        }
    });

    // Return a response indicating the trade request is in progress
    format!(
        "Sending `{}` a trade request in `The Bard's Theater #1` trading channel",
        in_game_id
    )
}

#[get("/claim_items/<in_game_id>/<discord_channel_id>/<discord_id>")]
fn claim_items(
    in_game_id: String,
    discord_channel_id: &str,
    discord_id: &str,
    enigo: &State<Arc<Mutex<Enigo>>>,
    bot_info: &State<Arc<Mutex<TradeBotInfo>>>,
    traders_container: &State<Arc<Mutex<TradersContainer>>>,
) -> String {
    {
        let mut traders = traders_container.lock().unwrap();
        traders.set_in_game_id_by_discord_info(in_game_id.as_str(), discord_id, discord_channel_id);
    }

    // Check the bot status before proceeding
    let info = bot_info.lock().unwrap();
    match info.ready {
        ReadyState::False => {
            update_status("TradeBot not ready").unwrap();
            return String::from("TradeBot not ready");
        }
        ReadyState::Starting => {
            update_status("TradeBot is starting. Please wait 2 minutes.").unwrap();
            return String::from("TradeBot is starting. Please wait 2 minutes.");
        }
        ReadyState::True => {
            update_status("TradeBot ready").unwrap();
        }
    }

    // Dereference `State` and clone the inner `Arc`.
    let enigo_cloned = enigo.inner().clone();
    let bot_info_cloned = bot_info.inner().clone();
    let traders_container_cloned = traders_container.inner().clone();
    let in_game_id_cloned = in_game_id.clone();

    // Spawning a new asynchronous task
    tokio::spawn(async move {
        // Using spawn_blocking to handle potential blocking/synchronous code
        let result = tokio::task::spawn_blocking(move || {
            match trading_functions::claim_items(
                enigo_cloned,
                bot_info_cloned,
                in_game_id_cloned.as_ref(),
                traders_container_cloned,
            ) {
                Ok(_) => return String::from("Trade successful!"),
                Err(err) => {
                    if err == String::from("No items left in escrow") {
                        return String::from("All items traded");
                    }
                    return err;
                }
            }
        })
        .await;

        tokio::task::yield_now().await;

        // Log the result or handle it further, based on requirements
        match result {
            Ok(s) => {
                let _ = update_status(s.as_str());
                println!("Trade result: {}", s);
            }
            Err(e) => {
                let _ = update_status(format!("{:?}", e).as_str());
                eprintln!("Trade error: {:?}", e)
            }
        }
    });

    // Return a response indicating the trade request is in progress
    format!(
        "Sending `{}` a trade request in `The Bard's Theater #1` trading channel",
        in_game_id
    )
}

#[get("/claim_gold/<in_game_id>/<discord_channel_id>/<discord_id>")]
fn claim_gold(
    in_game_id: String,
    discord_channel_id: &str,
    discord_id: &str,
    enigo: &State<Arc<Mutex<Enigo>>>,
    bot_info: &State<Arc<Mutex<TradeBotInfo>>>,
    traders_container: &State<Arc<Mutex<TradersContainer>>>,
) -> String {
    {
        let mut traders = traders_container.lock().unwrap();
        traders.set_in_game_id_by_discord_info(in_game_id.as_str(), discord_id, discord_channel_id);
    }

    // Check the bot status before proceeding
    let info = bot_info.lock().unwrap();
    match info.ready {
        ReadyState::False => {
            update_status("TradeBot not ready").unwrap();
            return String::from("TradeBot not ready");
        }
        ReadyState::Starting => {
            update_status("TradeBot is starting. Please wait 2 minutes.").unwrap();
            return String::from("TradeBot is starting. Please wait 2 minutes.");
        }
        ReadyState::True => {
            update_status("TradeBot ready").unwrap();
        }
    }

    let enigo_cloned = enigo.inner().clone();
    let bot_info_cloned = bot_info.inner().clone();
    let traders_container_cloned = traders_container.inner().clone();
    let in_game_id_cloned = in_game_id.clone();

    // Spawning a new asynchronous task
    tokio::spawn(async move {
        // Using spawn_blocking to handle potential blocking/synchronous code
        let result = tokio::task::spawn_blocking(move || {
            match trading_functions::claim_gold(
                enigo_cloned,
                bot_info_cloned,
                in_game_id_cloned.as_ref(),
                traders_container_cloned,
            ) {
                Ok(_) => return String::from("Trade successful!"),
                Err(err) => {
                    if err == String::from("No gold left in escrow") {
                        return String::from("All gold traded");
                    }
                    return err;
                }
            }
        })
        .await;

        tokio::task::yield_now().await;

        // Log the result or handle it further, based on requirements
        match result {
            Ok(s) => {
                let _ = update_status(s.as_str());
                println!("Trade result: {}", s);
            }
            Err(e) => {
                let _ = update_status(format!("{:?}", e).as_str());
                eprintln!("Trade error: {:?}", e)
            }
        }
    });

    // Return a response indicating the trade request is in progress
    format!(
        "Sending `{}` a trade request in `The Bard's Theater #1` trading channel",
        in_game_id
    )
}

#[get("/return_items/<in_game_id>/<discord_channel_id>/<discord_id>")]
fn return_items(
    in_game_id: String,
    discord_channel_id: &str,
    discord_id: &str,
    enigo: &State<Arc<Mutex<Enigo>>>,
    bot_info: &State<Arc<Mutex<TradeBotInfo>>>,
    traders_container: &State<Arc<Mutex<TradersContainer>>>,
) -> String {
    {
        let mut traders = traders_container.lock().unwrap();
        traders.set_in_game_id_by_discord_info(in_game_id.as_str(), discord_id, discord_channel_id);
    }

    // Check the bot status before proceeding
    let info = bot_info.lock().unwrap();
    match info.ready {
        ReadyState::False => {
            update_status("TradeBot not ready").unwrap();
            return String::from("TradeBot not ready");
        }
        ReadyState::Starting => {
            update_status("TradeBot is starting. Please wait 2 minutes.").unwrap();
            return String::from("TradeBot is starting. Please wait 2 minutes.");
        }
        ReadyState::True => {
            update_status("TradeBot ready").unwrap();
        }
    }

    // Dereference `State` and clone the inner `Arc`.
    let enigo_cloned = enigo.inner().clone();
    let bot_info_cloned = bot_info.inner().clone();
    let traders_container_cloned = traders_container.inner().clone();
    let in_game_id_cloned = in_game_id.clone();

    // Spawning a new asynchronous task
    tokio::spawn(async move {
        // Using spawn_blocking to handle potential blocking/synchronous code
        let result = tokio::task::spawn_blocking(move || {
            match trading_functions::return_items(
                enigo_cloned,
                bot_info_cloned,
                in_game_id_cloned.as_ref(),
                traders_container_cloned,
            ) {
                Ok(_) => return String::from("Trade successful!"),
                Err(err) => {
                    if err == String::from("No items left in escrow") {
                        return String::from("All items traded");
                    }
                    return err;
                }
            }
        })
        .await;

        tokio::task::yield_now().await;

        // Log the result or handle it further, based on requirements
        match result {
            Ok(s) => {
                let _ = update_status(s.as_str());
                println!("Trade result: {}", s);
            }
            Err(e) => {
                let _ = update_status(format!("{:?}", e).as_str());
                eprintln!("Trade error: {:?}", e)
            }
        }
    });

    // Return a response indicating the trade request is in progress
    format!(
        "Sending `{}` a trade request in `The Bard's Theater #1` trading channel",
        in_game_id
    )
}

#[get("/return_gold/<in_game_id>/<discord_channel_id>/<discord_id>")]
fn return_gold(
    in_game_id: String,
    discord_channel_id: &str,
    discord_id: &str,
    enigo: &State<Arc<Mutex<Enigo>>>,
    bot_info: &State<Arc<Mutex<TradeBotInfo>>>,
    traders_container: &State<Arc<Mutex<TradersContainer>>>,
) -> String {
    {
        let mut traders = traders_container.lock().unwrap();
        traders.set_in_game_id_by_discord_info(in_game_id.as_str(), discord_id, discord_channel_id);
    }

    // Check the bot status before proceeding
    let info = bot_info.lock().unwrap();
    match info.ready {
        ReadyState::False => {
            update_status("TradeBot not ready").unwrap();
            return String::from("TradeBot not ready");
        }
        ReadyState::Starting => {
            update_status("TradeBot is starting. Please wait 2 minutes.").unwrap();
            return String::from("TradeBot is starting. Please wait 2 minutes.");
        }
        ReadyState::True => {
            update_status("TradeBot ready").unwrap();
        }
    }

    let enigo_cloned = enigo.inner().clone();
    let bot_info_cloned = bot_info.inner().clone();
    let traders_container_cloned = traders_container.inner().clone();
    let in_game_id_cloned = in_game_id.clone();

    // Spawning a new asynchronous task
    tokio::spawn(async move {
        // Using spawn_blocking to handle potential blocking/synchronous code
        let result = tokio::task::spawn_blocking(move || {
            match trading_functions::return_gold(
                enigo_cloned,
                bot_info_cloned,
                in_game_id_cloned.as_ref(),
                traders_container_cloned,
            ) {
                Ok(_) => return String::from("Trade successful!"),
                Err(err) => {
                    if err == String::from("No gold left in escrow") {
                        return String::from("All gold traded");
                    }
                    return err;
                }
            }
        })
        .await;

        tokio::task::yield_now().await;

        // Log the result or handle it further, based on requirements
        match result {
            Ok(s) => {
                let _ = update_status(s.as_str());
                println!("Trade result: {}", s);
            }
            Err(e) => {
                let _ = update_status(format!("{:?}", e).as_str());
                eprintln!("Trade error: {:?}", e)
            }
        }
    });

    // Return a response indicating the trade request is in progress
    format!(
        "Sending `{}` a trade request in `The Bard's Theater #1` trading channel",
        in_game_id
    )
}

fn rocket() -> rocket::Rocket<rocket::Build> {
    // Create 2 instances of enigo because Enigo does not implement Copy.
    let enigo = Arc::new(Mutex::new(Enigo::new()));

    let bot_info = Arc::new(Mutex::new(TradeBotInfo {
        ready: ReadyState::True,
        id: "".to_string(),
    }));

    let traders_container = Arc::new(Mutex::new(TradersContainer::ActiveTraders(Vec::new())));

    match database_functions::populate_traders_from_db(&traders_container) {
        Ok(_) => println!("Populated trades containter!"),
        Err(err) => println!("Could not populate traders containter. Error:\n{}", err),
    }

    // Clone the Arc for use in main_func
    let bot_info_clone = bot_info.clone();

    // Spawn the open game function as a separate task
    /*
    tokio::spawn(async move {
        trading_functions::open_game_go_to_lobby(bot_info_clone).await;
    });
    */
    
    // Going back to lobby
    // This is only useful if the bot was restarted and not already in the lobby
    tokio::spawn(async move {
        trading_functions::return_to_lobby();
    });

    rocket::build()
        .manage(enigo) // Add the enigo as managed state
        .manage(bot_info) // Add the bot_info as managed state
        .manage(traders_container) // Add the traders_container as managed state
        .mount(
            "/",
            routes![
                gold_fee,
                deposit,
                claim_items,
                claim_gold,
                return_items,
                return_gold,
                restart
            ],
        )
}

#[rocket::main]
async fn main() {
    // Simply launch Rocket in the main function
    let rocket_instance = rocket();
    if let Err(err) = rocket_instance.launch().await {
        eprintln!("Rocket server error: {}", err);
    }
}

// One way file-based communication between in-game bot and discord bot
fn update_status(text_to_write: &str) -> io::Result<()> {
    let file_path = "../shared/ipc_communication.txt";

    write(file_path, text_to_write)?;

    Ok(())
}
