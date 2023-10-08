use std::str;
use std::sync::{Arc, Mutex};

use enigo::*;

// import Rocket
#[macro_use]
extern crate rocket;
use rocket::State;

mod database_functions;
mod enigo_functions;
mod trading_functions;

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
            TradersContainer::ActiveTraders(traders) => {
                traders.iter().find(|trader| trader.in_game_id == in_game_id)
            }
        }
    }

    fn update_gold_fee_status(&mut self, in_game_id: &str, new_status: bool) {
        match self {
            TradersContainer::ActiveTraders(traders) => {
                if let Some(trader) = traders.iter_mut().find(|trader| trader.in_game_id == in_game_id) {
                    trader.has_paid_gold_fee = new_status;
                }
            }
        }
    }
}

#[get("/gold_fee/<in_game_id>/<discord_channel_id>/<discord_id>")]
fn gold_fee(
    in_game_id: &str,
    discord_channel_id: &str,
    discord_id: &str,
    enigo: &State<Arc<Mutex<Enigo>>>,
    bot_info: &State<Arc<Mutex<TradeBotInfo>>>,
    traders_container: &State<Arc<Mutex<TradersContainer>>>,
) -> String {
    {
        let info = bot_info.lock().unwrap();
        match info.ready{
            ReadyState::False => return String::from("TradeBot not ready"),
            ReadyState::Starting => return String::from("TradeBot is starting. Please wait 2 minutes and try again."),
            ReadyState::True => println!("Going into trade!"),
            
        }
    } // Lock is released here as the MutexGuard goes out of scope

    let mut traders = traders_container.lock().unwrap();

    // Write the database part in python first and then come back and retrive it here.
    let item_links =
        database_functions::get_links_for_user(discord_channel_id, discord_id).unwrap();

    let trader = Trader {
        in_game_id: String::from(in_game_id),
        discord_channel_id: String::from(discord_channel_id),
        discord_id: String::from(discord_id),
        item_images: item_links.0,
        info_images: item_links.1,
        gold: 0,
        has_paid_gold_fee: false,
    };

    traders.append(trader);


    trading_functions::collect_gold_fee(enigo, bot_info, in_game_id, traders_container);

    format!("TradeBot ready\n{}", bot_info.lock().unwrap().id)
}

#[get("/trade_request/<in_game_id>")]
fn trade_request(
    in_game_id: &str,
    enigo: &State<Arc<Mutex<Enigo>>>,
    bot_info: &State<Arc<Mutex<TradeBotInfo>>>,
    traders_container: &State<Arc<Mutex<TradersContainer>>>,
) -> String {
    {
        let info = bot_info.lock().unwrap();
        match info.ready{
            ReadyState::False => return String::from("TradeBot not ready"),
            ReadyState::Starting => return String::from("TradeBot is starting. Please wait 2 minutes and try again."),
            ReadyState::True => println!("Going into trade!"),
            
        }
    } // Lock is released here as the MutexGuard goes out of scope

    trading_functions::complete_trade(enigo, bot_info, in_game_id, traders_container);

    format!("TradeBot ready\n{}", bot_info.lock().unwrap().id)
}

fn rocket() -> rocket::Rocket<rocket::Build> {
    // Create 2 instances of enigo because Enigo does not implement Copy.
    let enigo = Arc::new(Mutex::new(Enigo::new()));

    let bot_info = Arc::new(Mutex::new(TradeBotInfo {
        ready: ReadyState::False,
        id: "".to_string(),
    }));

    let traders_container = Arc::new(Mutex::new(TradersContainer::ActiveTraders(Vec::new())));

    // Clone the Arc for use in main_func
    let bot_info_clone = bot_info.clone();

    // Spawn the open game function as a separate task
    tokio::spawn(async move {
        trading_functions::open_game_go_to_lobby(bot_info_clone).await;
    });

    rocket::build()
        .manage(enigo) // Add the enigo as managed state
        .manage(bot_info) // Add the bot_info as managed state
        .manage(traders_container) // Add the traders_container as managed state
        .mount("/", routes![gold_fee, trade_request])
}

#[rocket::main]
async fn main() {
    // Simply launch Rocket in the main function
    let rocket_instance = rocket();
    if let Err(err) = rocket_instance.launch().await {
        eprintln!("Rocket server error: {}", err);
    }
}
