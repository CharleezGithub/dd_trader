use std::process::Command;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use std::fs::File;
use std::io;
use std::path::Path;

use reqwest;

use enigo::*;
use rand::Rng;
use rocket::State;

use crate::TradersContainer;
use crate::{database_functions, ReadyState, TradeBotInfo};

use crate::enigo_functions;

// Opens the windows search bar and searches for a given title and opens it
fn start_game(enigo: &mut Enigo, launcher_name: &str) {
    enigo.key_click(Key::Meta);
    sleep(Duration::from_millis(1000));
    enigo.key_sequence_parse(launcher_name);
    sleep(Duration::from_millis(2000));
    enigo.key_click(Key::Return);
}

// This function does the following:
// 1. Opens the blacksmith launcher and presses play
// 2. Goes into the lobby.
// 3. Changes the TradeBotInfo ready variable to true when ready.
pub async fn open_game_go_to_lobby(bot_info: Arc<Mutex<TradeBotInfo>>) {
    let enigo = Arc::new(Mutex::new(Enigo::new()));

    println!("Opening game!");
    {
        let mut bot_info = bot_info.lock().unwrap();
        bot_info.ready = ReadyState::Starting;
    }
    //tokio::time::sleep(tokio::time::Duration::from_secs(10000)).await;

    let mut enigo = enigo.lock().unwrap();

    // Minimizes all tabs so that only the game is opened. To avoid clicking on other tabs
    enigo.key_sequence_parse("{+META}m{-META}");

    // Start the launcher
    start_game(&mut enigo, "blacksmith");

    // Run the launcher play button detector
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/play.png")
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
        .arg("python_helpers/obj_detection.py")
        .arg("images/okay_start.png")
        .output()
        .expect("Failed to execute command");

    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Run the "Enter the lobby" button detector
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/enter_lobby.png")
        .output()
        .expect("Failed to execute command");

    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Now the bot is in the lobby "play" tab
    let mut info = bot_info.lock().unwrap();
    info.ready = ReadyState::True;
    info.id = String::from("Middleman2");
}

// It waits untill a trade request is sent by the discord bot
pub fn collect_gold_fee(
    enigo: &State<Arc<Mutex<Enigo>>>,
    bot_info: &State<Arc<Mutex<TradeBotInfo>>>,
    in_game_id: &str,
    traders_container: &State<Arc<Mutex<TradersContainer>>>,
) {
    let mut enigo = enigo.lock().unwrap();

    let info = bot_info.lock().unwrap();

    // If the bot is not ready then it will run the open game function
    // If the bot is starting then it will wait for the bot to be ready
    // If the bot is ready then it will continue as normal
    'wait_loop: loop {
        let bot_info_clone = bot_info.inner().clone();
        match info.ready {
            ReadyState::False => {
                tokio::spawn(async move {
                    open_game_go_to_lobby(bot_info_clone).await;
                });
            }
            ReadyState::Starting => sleep(Duration::from_secs(2)),
            ReadyState::True => break 'wait_loop,
        }
    }
    // Goes into the trading tab and connects to bards trade post.
    // Why bard? Because it has the least amount of active traders and therefore not as demanding to be in.
    // Run the "Trade" tab detector
    let _ = send_trade_request(in_game_id);

    // Check if user has put in 50 gold for the trade fee
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/gold_fee2.png")
        .arg("S")
        .output();

    match output {
        Ok(_) => println!("User put in the gold fee."),
        Err(_) => {
            println!("User did not put in gold fee..");
            return_to_lobby();
            return;
        }
    }

    // Click the checkbox
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/trade_checkbox.png")
        .output()
        .expect("Failed to execute command");

    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Double check that the total gold is still the same in the trade confirmation window
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/gold_fee_double_check.png")
        .arg("S")
        .output();

    match output {
        Ok(_) => println!("User put in the gold fee."),
        Err(_) => {
            println!("User did not put in gold fee..");
            return_to_lobby();
            return;
        }
    }

    // Click the magnifying glasses on top of the items
    let output = Command::new("python")
        .arg("python_helpers/inspect_items.py")
        .output()
        .expect("Failed to execute command");

    // Convert the output bytes to a string
    let output_str = str::from_utf8(&output.stdout).unwrap().trim();

    // Split the string on newlines to get the list of coordinates
    let coords: Vec<&str> = output_str.split('\n').collect();

    // Now, coords contains each of the coordinates
    for coord_str in coords.iter() {
        let coord: Vec<i32> = coord_str
            .split_whitespace()
            .map(|s| s.parse().expect("Failed to parse coordinate"))
            .collect();

        if coord.len() == 4 {
            let (x1, y1, x2, y2) = (coord[0], coord[1], coord[2], coord[3]);

            let mut rng = rand::thread_rng();

            // Salt the pixels so that it does not click the same pixel every time.
            let salt = rng.gen_range(-9..9);

            // Gets the middle of the detected play button and clicks it
            let middle_point_x = ((x2 - x1) / 2) + x1 + salt;
            let middle_point_y = ((y2 - y1) / 2) + y1 + salt;

            match enigo_functions::click_buton_right_direct(
                &mut enigo,
                middle_point_x,
                middle_point_y,
                true,
                true,
                0,
                0,
            ) {
                Ok(_) => println!("Successfully clicked button!"),
                Err(err) => println!("Got error while trying to click button: {:?}", err),
            }
        }
    }

    // Click the final checkpoint to get the 50 gold fee
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/trade_checkbox.png")
        .output()
        .expect("Failed to execute command");

    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // When paid, set has_paid_gold_fee to true
    let mut traders = traders_container.lock().unwrap();
    let trader = traders.get_trader_by_in_game_id(in_game_id);

    // Check if trader exists
    match trader {
        Some(trader) => {
            match database_functions::set_gold_fee_status(
                trader.discord_channel_id.as_str(),
                trader.discord_id.as_str(),
                true,
            ) {
                Ok(_) => println!("Succesfully updated gold fee status!"),
                Err(err) => println!("Could not update gold status: Error \n{}", err),
            }
        }
        None => println!("Trader not found"),
    }

    // Make a copy of trader discord id. Else it would use traders as both mutable and imutable.
    let trader_discord_id = trader.unwrap().discord_id.as_str();
    let trader_discord_id_copy: String = String::from(trader_discord_id);
    traders.update_gold_fee_status(trader_discord_id_copy.as_str(), true);
}

pub fn complete_trade(
    enigo: &State<Arc<Mutex<Enigo>>>,
    bot_info: &State<Arc<Mutex<TradeBotInfo>>>,
    in_game_id: &str,
    traders_container: &State<Arc<Mutex<TradersContainer>>>,
) {
    let mut enigo = enigo.lock().unwrap();

    let info = bot_info.lock().unwrap();

    // If the bot is not ready then it will run the open game function
    // If the bot is starting then it will wait for the bot to be ready
    // If the bot is ready then it will continue as normal
    'wait_loop: loop {
        let bot_info_clone = bot_info.inner().clone();
        match info.ready {
            ReadyState::False => {
                tokio::spawn(async move {
                    open_game_go_to_lobby(bot_info_clone).await;
                });
            }
            ReadyState::Starting => sleep(Duration::from_secs(2)),
            ReadyState::True => break 'wait_loop,
        }
    }

    // Get the trader with that in-game name
    let traders = traders_container.lock().unwrap();
    let trader = traders.get_trader_by_in_game_id(in_game_id);

    // Get channel and discord id
    let channel_id = trader.unwrap().discord_channel_id.as_str();
    let discord_id = trader.unwrap().discord_id.as_str();

    let has_paid_fee = database_functions::has_paid_fee(channel_id, discord_id).unwrap();

    if !has_paid_fee {
        return;
    }

    // Go into the trading tab and send a trade to the trader. Exact same as before with the gold fee.
    let _ = send_trade_request(trader.unwrap().in_game_id.as_str());

    // Now we are in the trading window with the trader

    // Loop through the items in the trader struct for this trader and use obj detection to check if the item is present
    // If item is present then add it to list. Once it cannot find any more items in the trading window (Wait at least 30 seconds after detection an item so that the trader has time to put in the stuff) it should accept the items

    // Download 1 image set into temp_images folder at a time and check for a match
    let info_vec = &trader.unwrap().info_images;
    let item_vec = &trader.unwrap().item_images;

    // For each image pair. Download the pair and if there is a matching pair in the trading window, add it to list in memory.
    // After trading successfully and double checking in inspect window, change status to "in escrow" for the traded items in the database.
    let mut trading_window_items = Vec::new();

    for item in item_vec.iter() {
        match download_image(&item, "temp_images/item/image.png") {
            Ok(_) => println!("Successfully downloaded item image"),
            Err(err) => {
                println!("Could not download image. Error \n{}", err);
                return;
            }
        }

        let output = Command::new("python")
            .arg("python_helpers/multi_obj_detection.py")
            .arg("temp_images/item/image.png")
            .output()
            .expect("Failed to execute command");

        // Convert the output bytes to a string
        let output_str = str::from_utf8(&output.stdout).unwrap().trim();

        // Split the string on newlines to get the list of coordinates
        let coords: Vec<&str> = output_str.split('\n').collect();

        // Now, coords contains each of the coordinates
        for coord_str in coords.iter() {
            let coord: Vec<i32> = coord_str
                .split_whitespace()
                .map(|s| s.parse().expect("Failed to parse coordinate"))
                .collect();

            if coord.len() == 4 {
                let (x1, y1, x2, y2) = (coord[0], coord[1], coord[2], coord[3]);

                let mut rng = rand::thread_rng();

                // Salt the pixels so that it does not click the same pixel every time.
                let salt = rng.gen_range(-9..9);

                // Gets the middle of the detected play button and clicks it
                let middle_point_x = ((x2 - x1) / 2) + x1 + salt;
                let middle_point_y = ((y2 - y1) / 2) + y1 + salt;

                match enigo_functions::move_to_location_fast(
                    &mut enigo,
                    middle_point_x,
                    middle_point_y,
                ) {
                    Ok(_) => println!("Successfully moved to this location!"),
                    Err(err) => println!("Got error while trying to move cursor: {:?}", err),
                }

                // Tries to match every info image with the item and if there is a match then it will add it to the temporary vector variable.
                for info_image in info_vec.iter() {
                    match download_image(info_image, "temp_images/info/image.png") {
                        Ok(_) => println!("Successfully downloaded info image"),
                        Err(err) => {
                            println!("Could not download image. Error \n{}", err);
                            return;
                        }
                    }

                    // SHOULD USE A VERSION OF OBJ DETECTION WITH A FASTER TIMEOUT. So that it wont wait for 4 minutes of there is no match
                    let output = Command::new("python")
                        .arg("python_helpers/obj_detection.py")
                        .arg("temp_images/info/item.png")
                        .output();

                    match output {
                        Ok(_) => {
                            println!("Found match!");
                            trading_window_items.push((info_image, item));
                        }
                        Err(_) => println!("No match. Checking next..."),
                    }
                }
            }
        }
    }

    // Make copy to use for later
    let trading_window_items_clone = trading_window_items.clone();
    // Accept trade
    // Click the checkbox
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/trade_checkbox.png")
        .output()
        .expect("Failed to execute command");

    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Click the magnifying glasses on top of the items
    let output = Command::new("python")
        .arg("python_helpers/inspect_items.py")
        .output()
        .expect("Failed to execute command");

    // Convert the output bytes to a string
    let output_str = str::from_utf8(&output.stdout).unwrap().trim();

    // Split the string on newlines to get the list of coordinates
    let coords: Vec<&str> = output_str.split('\n').collect();

    // Now, coords contains each of the coordinates
    for coord_str in coords.iter() {
        let coord: Vec<i32> = coord_str
            .split_whitespace()
            .map(|s| s.parse().expect("Failed to parse coordinate"))
            .collect();

        if coord.len() == 4 {
            let (x1, y1, x2, y2) = (coord[0], coord[1], coord[2], coord[3]);

            let mut rng = rand::thread_rng();

            // Salt the pixels so that it does not click the same pixel every time.
            let salt = rng.gen_range(-9..9);

            // Gets the middle of the detected play button and clicks it
            let middle_point_x = ((x2 - x1) / 2) + x1 + salt;
            let middle_point_y = ((y2 - y1) / 2) + y1 + salt;

            match enigo_functions::click_buton_right_direct(
                &mut enigo,
                middle_point_x,
                middle_point_y,
                true,
                true,
                0,
                0,
            ) {
                Ok(_) => println!("Successfully clicked button!"),
                Err(err) => println!("Got error while trying to click button: {:?}", err),
            }
        }
    }

    // Now double check that the items are still in the trading window by itterating through the trading window vector and hovering over all the items again.
    // Pair is (info, item)

    // To store indices to remove
    let mut indices_to_remove = Vec::new();

    for (index, pair) in trading_window_items.iter().enumerate() {
        match download_image(&pair.1, "temp_images/item/image.png") {
            Ok(_) => println!("Successfully downloaded item image"),
            Err(err) => {
                println!("Could not download image. Error \n{}", err);
                return;
            }
        }

        // Handling output and avoiding temporary value drop issue
        let output_result = Command::new("python")
            .arg("python_helpers/multi_obj_detection.py")
            .arg("temp_images/item/image.png")
            .output();

        match output_result {
            Ok(output) => {
                let output_bytes = output.stdout;
                let output_str = str::from_utf8(&output_bytes).unwrap().trim();
                let coords: Vec<&str> = output_str.split('\n').collect();

                // Now, coords contains each of the coordinates
                for coord_str in coords.iter() {
                    let coord: Vec<i32> = coord_str
                        .split_whitespace()
                        .map(|s| s.parse().expect("Failed to parse coordinate"))
                        .collect();

                    if coord.len() == 4 {
                        let (x1, y1, x2, y2) = (coord[0], coord[1], coord[2], coord[3]);

                        let mut rng = rand::thread_rng();

                        // Salt the pixels so that it does not click the same pixel every time.
                        let salt = rng.gen_range(-9..9);

                        // Gets the middle of the detected play button and clicks it
                        let middle_point_x = ((x2 - x1) / 2) + x1 + salt;
                        let middle_point_y = ((y2 - y1) / 2) + y1 + salt;

                        match enigo_functions::move_to_location_fast(
                            &mut enigo,
                            middle_point_x,
                            middle_point_y,
                        ) {
                            Ok(_) => println!("Successfully moved to this location!"),
                            Err(err) => {
                                println!("Got error while trying to move cursor: {:?}", err)
                            }
                        }

                        // Tries to match every info image with the item and if there is a match then it will add it to the temporary vector variable.
                        for info_image in info_vec.iter() {
                            match download_image(info_image, "temp_images/info/image.png") {
                                Ok(_) => println!("Successfully downloaded info image"),
                                Err(err) => {
                                    println!("Could not download image. Error \n{}", err);
                                    return;
                                }
                            }

                            // SHOULD USE A VERSION OF OBJ DETECTION WITH A FASTER TIMEOUT. So that it won't wait for 4 minutes if there is no match
                            let output = Command::new("python")
                                .arg("python_helpers/obj_detection.py")
                                .arg("temp_images/info/item.png")
                                .output();

                            match output {
                                Ok(_) => {
                                    println!("Found match!");
                                    // Recording the index to remove after the loop
                                    indices_to_remove.push(index);
                                }
                                Err(_) => println!("No match. Checking next..."),
                            }
                        }
                    }
                }
            }
            Err(_) => {
                println!("Could not find item. Cancelling trade and going to lobby..");
                // GO TO LOBBY
                return_to_lobby();
                return;
            }
        }
    }

    // Removing items from trading_window_items using the indices recorded
    for index in indices_to_remove.into_iter().rev() {
        trading_window_items.remove(index);
    }

    // Check if trading_window_items is empty
    if trading_window_items.is_empty() {
        println!("The trading_window_items vector is empty.");
    } else {
        println!("The trading_window_items vector is not empty. Cancelling the trade!");
        return_to_lobby();
        return;
    }
    for pair in trading_window_items_clone.iter() {
        match database_functions::set_item_status_by_urls(pair.1, pair.0, "in escrow") {
            Ok(_) => println!("Updated item status!"),
            Err(err) => println!("Error updating item status. Error: \n{}", err),
        }
    }
}

// Collect items function
pub fn collect_trade(
    enigo: &State<Arc<Mutex<Enigo>>>,
    bot_info: &State<Arc<Mutex<TradeBotInfo>>>,
    in_game_id: &str,
    traders_container: &State<Arc<Mutex<TradersContainer>>>,
) {
    let mut enigo = enigo.lock().unwrap();

    let info = bot_info.lock().unwrap();

    // If the bot is not ready then it will run the open game function
    // If the bot is starting then it will wait for the bot to be ready
    // If the bot is ready then it will continue as normal
    'wait_loop: loop {
        let bot_info_clone = bot_info.inner().clone();
        match info.ready {
            ReadyState::False => {
                tokio::spawn(async move {
                    open_game_go_to_lobby(bot_info_clone).await;
                });
            }
            ReadyState::Starting => sleep(Duration::from_secs(2)),
            ReadyState::True => break 'wait_loop,
        }
    }

    // Get the trader with that in-game name
    let traders = traders_container.lock().unwrap();
    let trader = traders.get_trader_by_in_game_id(in_game_id).unwrap();

    let trader_discord_id = &trader.discord_id;
    let trader_channel_id = &trader.discord_channel_id;

    // Find the other trader in the same trade as the trader.
    // This is done so that we can search for the items that the other person has traded to the bot so that the trader can get the other traders items and not their own back.
    let other_trader = traders.get_other_trader_in_channel(&trader_discord_id, &trader_channel_id);

    match send_trade_request(in_game_id) {
        Ok(_) => println!("Player accepted trade request"),
        Err(_) => {
            println!("Player declined request. Going back to lobby.");
            return;
        }
    }
    // Now we are in the trading window
    // It should find matches in both the inventory and the stash and add them to the trading window.

    // These 2 vectors store the traders items. It loops through these and find pairs and adds them to the trade.
    let info_vec = &other_trader.unwrap().info_images;
    let item_vec = &other_trader.unwrap().item_images;

    // Store the items that made it through in this vector
    // Then when the trade is done loop through the list and change their status from "in escrow" to "traded"
    // (Info, item)
    let mut in_window_items = Vec::new();

    // For each image pair. Download the pair and if there is a matching pair in the stash or inventory, add it to the trading window.
    let mut item_limit = 10;
    'add_items: for item in item_vec.iter() {
        if item_limit <= 0 {
            break 'add_items;
        }
        match download_image(&item, "temp_images/item/image.png") {
            Ok(_) => println!("Successfully downloaded item image"),
            Err(err) => {
                println!("Could not download image. Error \n{}", err);
                return;
            }
        }

        // Convert the output bytes to a string
        let output_str = {
            let output = Command::new("python")
                .arg("python_helpers/multi_obj_detection.py")
                .arg("temp_images/item/image.png")
                .output()
                .expect("Failed to execute command");

            str::from_utf8(&output.stdout).unwrap().trim().to_string()
        };

        // If it could not detect any items in the inventory then go to stash and try again
        let output_str = if output_str == "Could not detect" {
            let output_stash = Command::new("python")
                .arg("python_helpers/obj_detection.py")
                .arg("images/stash.png")
                .output()
                .expect("Failed to execute command");

            match enigo_functions::click_buton(&mut enigo, output_stash, true, 0, 0) {
                Ok(_) => println!("Successfully clicked button!"),
                Err(err) => println!("Got error while trying to click button: {:?}", err),
            }

            let output_retry = Command::new("python")
                .arg("python_helpers/multi_obj_detection.py")
                .arg("temp_images/item/image.png")
                .output()
                .expect("Failed to execute command");

            // Convert the output bytes to a string
            str::from_utf8(&output_retry.stdout)
                .unwrap()
                .trim()
                .to_string()
        } else {
            output_str
        };

        if output_str == "Could not detect" {
            return_to_lobby();
            return;
        }

        // Split the string on newlines to get the list of coordinates
        let coords: Vec<&str> = output_str.split('\n').collect();

        // Now, coords contains each of the coordinates
        for coord_str in coords.iter() {
            let coord: Vec<i32> = coord_str
                .split_whitespace()
                .map(|s| s.parse().expect("Failed to parse coordinate"))
                .collect();

            if coord.len() == 4 {
                let (x1, y1, x2, y2) = (coord[0], coord[1], coord[2], coord[3]);

                let mut rng = rand::thread_rng();

                // Salt the pixels so that it does not click the same pixel every time.
                let salt = rng.gen_range(-9..9);

                // Gets the middle of the detected play button and clicks it
                let middle_point_x = ((x2 - x1) / 2) + x1 + salt;
                let middle_point_y = ((y2 - y1) / 2) + y1 + salt;

                match enigo_functions::move_to_location_fast(
                    &mut enigo,
                    middle_point_x,
                    middle_point_y,
                ) {
                    Ok(_) => println!("Successfully moved to this location!"),
                    Err(err) => println!("Got error while trying to move cursor: {:?}", err),
                }

                // Tries to match every info image with the item and if there is a match then it will add it to the temporary vector variable.
                for info_image in info_vec.iter() {
                    match download_image(info_image, "temp_images/info/image.png") {
                        Ok(_) => println!("Successfully downloaded info image"),
                        Err(err) => {
                            println!("Could not download image. Error \n{}", err);
                            return;
                        }
                    }

                    // SHOULD USE A VERSION OF OBJ DETECTION WITH A FASTER TIMEOUT. So that it wont wait for 4 minutes of there is no match
                    let output = Command::new("python")
                        .arg("python_helpers/obj_detection.py")
                        .arg("temp_images/info/item.png")
                        .output();

                    let output_unwrapped = output.unwrap(); // Bind `Output` to a variable to extend its lifetime
                    let output_str = str::from_utf8(&output_unwrapped.stdout).unwrap().trim();

                    if output_str != "Could not detect" {
                        println!("Found match!");
                        enigo.key_down(Key::Shift);
                        enigo.mouse_click(MouseButton::Left);
                        enigo.key_up(Key::Shift);
                        in_window_items.push((info_image, item));
                        item_limit += -1;
                    } else {
                        println!("No match. Checking next...");
                    }
                }
            }
        }
    }

    // Click checkbox to get into the confirmation trading window.
    // Click the checkbox
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/trade_checkbox.png")
        .output()
        .expect("Failed to execute command");

    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Now the bot is in the double check trade window box.
    // Click the magnifying glasses on top of the items
    let output = Command::new("python")
        .arg("python_helpers/inspect_items.py")
        .output()
        .expect("Failed to execute command");

    // Convert the output bytes to a string
    let output_str = str::from_utf8(&output.stdout).unwrap().trim();

    // Split the string on newlines to get the list of coordinates
    let coords: Vec<&str> = output_str.split('\n').collect();

    // Now, coords contains each of the coordinates
    for coord_str in coords.iter() {
        let coord: Vec<i32> = coord_str
            .split_whitespace()
            .map(|s| s.parse().expect("Failed to parse coordinate"))
            .collect();

        if coord.len() == 4 {
            let (x1, y1, x2, y2) = (coord[0], coord[1], coord[2], coord[3]);

            let mut rng = rand::thread_rng();

            // Salt the pixels so that it does not click the same pixel every time.
            let salt = rng.gen_range(-9..9);

            // Gets the middle of the detected play button and clicks it
            let middle_point_x = ((x2 - x1) / 2) + x1 + salt;
            let middle_point_y = ((y2 - y1) / 2) + y1 + salt;

            match enigo_functions::click_buton_right_direct(
                &mut enigo,
                middle_point_x,
                middle_point_y,
                true,
                true,
                0,
                0,
            ) {
                Ok(_) => println!("Successfully clicked button!"),
                Err(err) => println!("Got error while trying to click button: {:?}", err),
            }
        }
    }

    // Make an imuttable clone of in_window_items for enumeration to avoid borrow checker error.
    let in_window_items_clone = in_window_items.clone();

    // Now check what items made it into the trading window by going through the list of items again and adding those who match in the confirmation window to a list.
    // When there is no more items to add, click the checkbox and if the trade goes through, change the status of those items to "traded"

    // Pair is (info, item)
    for (index, pair) in in_window_items_clone.iter().enumerate() {
        match download_image(&pair.1, "temp_images/item/image.png") {
            Ok(_) => println!("Successfully downloaded item image"),
            Err(err) => {
                println!("Could not download image. Error \n{}", err);
                return;
            }
        }

        // Need to make sure that the detection script only sees the trading box and does not detect to the sides.
        // Because the inventory/stash is still visable on this screen so the screenshot that the bot takes needs to be narrowed to only the trading window.
        todo!();

        // Handling output and avoiding temporary value drop issue
        let output_result = Command::new("python")
            .arg("python_helpers/multi_obj_detection.py")
            .arg("temp_images/item/image.png")
            .output();

        match output_result {
            Ok(output) => {
                let output_bytes = output.stdout;
                let output_str = str::from_utf8(&output_bytes).unwrap().trim();
                let coords: Vec<&str> = output_str.split('\n').collect();

                // Now, coords contains each of the coordinates
                for coord_str in coords.iter() {
                    let coord: Vec<i32> = coord_str
                        .split_whitespace()
                        .map(|s| s.parse().expect("Failed to parse coordinate"))
                        .collect();

                    if coord.len() == 4 {
                        let (x1, y1, x2, y2) = (coord[0], coord[1], coord[2], coord[3]);

                        let mut rng = rand::thread_rng();

                        // Salt the pixels so that it does not click the same pixel every time.
                        let salt = rng.gen_range(-9..9);

                        // Gets the middle of the detected play button and clicks it
                        let middle_point_x = ((x2 - x1) / 2) + x1 + salt;
                        let middle_point_y = ((y2 - y1) / 2) + y1 + salt;

                        match enigo_functions::move_to_location_fast(
                            &mut enigo,
                            middle_point_x,
                            middle_point_y,
                        ) {
                            Ok(_) => println!("Successfully moved to this location!"),
                            Err(err) => {
                                println!("Got error while trying to move cursor: {:?}", err)
                            }
                        }

                        // Tries to match every info image with the item and if there is a match then it will add it to the temporary vector variable.
                        for info_image in info_vec.iter() {
                            match download_image(info_image, "temp_images/info/image.png") {
                                Ok(_) => println!("Successfully downloaded info image"),
                                Err(err) => {
                                    println!("Could not download image. Error \n{}", err);
                                    return;
                                }
                            }

                            // SHOULD USE A VERSION OF OBJ DETECTION WITH A FASTER TIMEOUT. So that it won't wait for 4 minutes if there is no match
                            let output = Command::new("python")
                                .arg("python_helpers/obj_detection.py")
                                .arg("temp_images/info/item.png")
                                .output();

                            match output {
                                Ok(_) => {
                                    println!("Found match!");
                                }
                                // Might not work...
                                Err(_) => {
                                    println!("No match. Checking next...");
                                    in_window_items.remove(index);
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => {
                println!("Could not find item. Cancelling trade and going to lobby..");
                // GO TO LOBBY
                return_to_lobby();
                return;
            }
        }
    }

    // Check if trading_window_items is empty
    if in_window_items.is_empty() {
        println!("No matches where found! Going back to lobby");
        return_to_lobby();
        return;
    }
    // If the in_window_items is not emtpy then change the status of those images from "in escrow" to "traded"
    else {
        // The bot ensures that the trade went through by making sure that it is the last link in the trade.
        // The bot waits for the trader to accept the trade by clicking the checkmark before the bot itself does.
        // Right as the trader clicks the button, the bot does as well, completing the trade for centain.
        // SHOULD USE A VERSION OF OBJ DETECTION WITH A FASTER TIMEOUT. So that it won't wait for 4 minutes if there is no match
        let output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/trader_ready.png")
            .output();

        match output {
            Ok(_) => {
                println!("User accepted trade!");
                // Click the checkbox fast so that the other trader does not have time to decline in order to try to trick the bot.
                let output = Command::new("python")
                    .arg("python_helpers/obj_detection.py")
                    .arg("images/trade_checkbox.png")
                    .output()
                    .expect("Failed to execute command");

                // Convert the output into 4 coordinates and get the middle point of those.
                // Then use the move_to_location_fast function to quickly move to the checkbox and click it
                // Convert the output bytes to a string
                let output_str = str::from_utf8(&output.stdout).unwrap().trim();

                // Split the string on newlines to get the list of coordinates
                let coords: Vec<&str> = output_str.split('\n').collect();

                // Now, coords contains each of the coordinates
                for coord_str in coords.iter() {
                    let coord: Vec<i32> = coord_str
                        .split_whitespace()
                        .map(|s| s.parse().expect("Failed to parse coordinate"))
                        .collect();

                    if coord.len() == 4 {
                        let (x1, y1, x2, y2) = (coord[0], coord[1], coord[2], coord[3]);

                        let mut rng = rand::thread_rng();

                        // Salt the pixels so that it does not click the same pixel every time.
                        let salt = rng.gen_range(-9..9);

                        // Gets the middle of the detected play button and clicks it
                        let middle_point_x = ((x2 - x1) / 2) + x1 + salt;
                        let middle_point_y = ((y2 - y1) / 2) + y1 + salt;
                        
                        // Now move to the middlepoint
                        match enigo_functions::move_to_location_fast(&mut enigo, middle_point_x, middle_point_y) {
                            Ok(_) => println!("Successfully clicked button!"),
                            Err(err) => println!("Got error while trying to click button: {:?}", err),
                        }

                        enigo.mouse_click(MouseButton::Left);
                    }

                }

            }
            // Might not work...
            Err(_) => {
                println!("User did not accept trade.");
                // GO TO LOBBY
                return_to_lobby();
                return;
            }
        }

        println!("Changing the items statuses from 'in escrow' to 'traded'!");
        for (info_url, item_url) in in_window_items {
            match database_functions::set_item_status_by_urls(item_url, info_url, "traded") {
                Ok(_) => println!("Changed the item status for 1 item!"),
                Err(err) => println!("Got error while changing item status. Error: \n{}", err),
            }
        }
    }
}

fn send_trade_request(in_game_id: &str) -> Result<&str, &str> {
    let mut enigo = Enigo::new();

    // Goes into the trading tab and connects to bards trade post.
    // Why bard? Because it has the least amount of active traders and therefore not as demanding to be in.
    // Run the "Trade" tab detector
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/trade_tab.png")
        .output()
        .expect("Failed to execute command");

    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Now enter bards trading post
    // Run the "bard_trade" button detector
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/bard_trade.png")
        .output()
        .expect("Failed to execute command");

    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    //It now sends a trade to the player
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/find_id.png")
        .output()
        .expect("Failed to execute command");

    // Search after the trader in the trade tab
    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    let user_is_in_trade: bool;

    // Type in the name of the trader
    let in_game_id_lower = in_game_id.to_lowercase();
    let in_game_id_lower_str_red: &str = &in_game_id_lower;
    enigo.key_sequence_parse(in_game_id_lower_str_red);

    // This runs the obj_detection script which tries to find the trade button.
    // If the person is not in the game, then there will be no trade button to press.
    // The obj_detection script runs for 4 minutes

    // Clicks directly on the first person below the bot, which should be the player to trade with.
    match enigo_functions::click_buton_right_direct(&mut enigo, 1824, 312, true, false, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Send a trade request
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/trade_send_request.png")
        .output();

    user_is_in_trade = match &output {
        Ok(_) => true,
        Err(_) => false,
    };
    if user_is_in_trade {
        match enigo_functions::click_buton(&mut enigo, output.unwrap(), true, 0, 0) {
            Ok(_) => println!("Successfully clicked button!"),
            Err(err) => println!("Got error while trying to click button: {:?}", err),
        }
    }
    // Else go back to main window and return.
    else {
        return_to_lobby();
        return Err("Trader declined request");
    }
    Ok("User accepted trade")
}

fn return_to_lobby() {
    let mut enigo = Enigo::new();

    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/play_tab.png")
        .output()
        .expect("Failed to execute command");

    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }
    return;
}

fn download_image(url: &str, save_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure the 'temp_images' directory exists
    if !Path::new("temp_images").exists() {
        std::fs::create_dir("temp_images")?;
    }

    // Perform a blocking HTTP GET request
    let response = reqwest::blocking::get(url)?;

    // Ensure the request was successful
    if response.status().is_success() {
        // Open a file to write the image data
        let mut file = File::create(save_path)?;

        // Copy the response data to the file
        let response_body = response.bytes()?;
        io::copy(&mut response_body.as_ref(), &mut file)?;

        println!("Image downloaded to '{}'", save_path);
    } else {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "Failed to download image",
        )));
    }

    Ok(())
}
