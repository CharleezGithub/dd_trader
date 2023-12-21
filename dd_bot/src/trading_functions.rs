use std::process::Command;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use std::fs;
use std::fs::File;
use std::io;
use std::panic;
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
pub async fn open_game_go_to_lobby(bot_info: Arc<Mutex<TradeBotInfo>>, start_launcher: bool) {
    let enigo = Arc::new(Mutex::new(Enigo::new()));

    println!("Opening game!");
    {
        let mut bot_info = bot_info.lock().unwrap();
        bot_info.ready = ReadyState::Starting;
    }
    //tokio::time::sleep(tokio::time::Duration::from_secs(10000)).await;

    let mut enigo = enigo.lock().unwrap();

    // If the launcher is already open, for example if we are restarting the game, then we do not need to open the launcher again.
    if start_launcher {
        // Minimizes all tabs so that only the game is opened. To avoid clicking on other tabs
        enigo.key_sequence_parse("{+META}m{-META}");

        // Start the launcher
        start_game(&mut enigo, "blacksmith");
    }

    // Quickly check if the game needs to update
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/update.png")
        .arg("SF")
        .output()
        .expect("Failed to execute command");

    // Convert the output bytes to a string
    let output_str = str::from_utf8(&output.stdout).unwrap().trim();

    // If the update menu was found then click the update button
    if output_str != "Could not find" {
        match enigo_functions::click_buton(&mut enigo, output, false, 0, 0) {
            Ok(_) => println!("Successfully clicked button!"),
            Err(err) => println!("Got error while trying to click button: {:?}", err),
        }
    }

    // Run the launcher play button detector
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/play.png")
        .arg("L")
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
    discord_id: &str,
    discord_channel_id: &str,
    traders_container: &State<Arc<Mutex<TradersContainer>>>,
) -> Result<String, String> {
    let result_catch_panic = panic::catch_unwind(|| {
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
                        open_game_go_to_lobby(bot_info_clone, true).await;
                    });
                }
                ReadyState::Starting => sleep(Duration::from_secs(2)),
                ReadyState::True => break 'wait_loop,
            }
        }
        // Goes into the trading tab and connects to bards trade post.
        // Why bard? Because it has the least amount of active traders and therefore not as demanding to be in.
        // Run the "Trade" tab detector
        match send_trade_request(in_game_id) {
            Ok(_) => println!("Player accepted trade request"),
            Err(_) => {
                return_to_lobby();
                return Err(String::from("Player declined request"));
            }
        }

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
                return Err(String::from("User did not put in gold fee"));
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
                return Err(String::from("User did not put in gold fee"));
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
            let mut rng = rand::thread_rng();

            if *coord_str == "Could not detect" || *coord_str == "" {
                println!("Counld not find item");
                // Moving away from items for obj detection purposes.
                match enigo_functions::move_to_location_fast(
                    &mut enigo,
                    rng.gen_range(25..50),
                    rng.gen_range(200..300),
                    true,
                ) {
                    Ok(_) => println!("Successfully moved to this location!"),
                    Err(err) => println!("Got error while trying to move cursor: {:?}", err),
                }

                // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
                enigo.mouse_click(MouseButton::Left);
                continue;
            }
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

                match enigo_functions::click_buton_direct(
                    &mut enigo,
                    middle_point_x,
                    middle_point_y,
                    true,
                    false,
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

        match database_functions::set_gold_fee_status(discord_channel_id, discord_id, true) {
            Ok(_) => println!("Succesfully updated gold fee status!"),
            Err(err) => println!("Could not update gold status: Error \n{}", err),
        }

        // Make a copy of trader discord id. Else it would use traders as both mutable and imutable.
        traders.update_gold_fee_status(discord_id, true);

        return_to_lobby();
        return Ok(String::from("Successfully collected fee!"));
    });
    match result_catch_panic {
        Ok(ok_result) => {
            match ok_result {
                Ok(message) => return Ok(message),
                Err(err) => {
                    return_to_lobby();
                    // Handle the string error here, maybe log it or convert it to your error type
                    return Err(err);
                }
            }
        }
        Err(panic_error) => {
            // This branch handles the case where panic::catch_unwind caught a panic
            // You can log the panic information, perform cleanup, or return an appropriate error
            // Convert panic_error (Box<dyn Any + Send>) to a suitable error type if needed
            return_to_lobby();
            // Return a generic error message
            return Err(format!(
                "Got error while trying to unwrap panic. Error: {:?}",
                panic_error
            ));
        }
    }
}

pub fn deposit(
    enigo: &State<Arc<Mutex<Enigo>>>,
    bot_info: &State<Arc<Mutex<TradeBotInfo>>>,
    in_game_id: &str,
    traders_container: &State<Arc<Mutex<TradersContainer>>>,
) -> Result<String, String> {
    let result_catch_panic = panic::catch_unwind(|| {
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
                        open_game_go_to_lobby(bot_info_clone, true).await;
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
            return Err(String::from("User has not yet paid the gold fee"));
        }

        // Go into the trading tab and send a trade to the trader. Exact same as before with the gold fee.
        match send_trade_request(trader.unwrap().in_game_id.as_str()) {
            Ok(_) => println!("Player accepted trade request"),
            Err(_) => {
                println!("Player declined request. Going back to lobby.");
                return_to_lobby();
                return Err(String::from("Player declined trade request"));
            }
        }

        // Now we are in the trading window with the trader

        // Wait for the trader to be ready and then accept the trade
        let output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/trader_ready.png")
            .output();

        match output {
            Ok(_) => println!("User did accept the trade"),
            Err(_) => {
                println!("User did not accept trade");
                return_to_lobby();
                return Err(String::from("User did not accept trade"));
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

        // Wait for trading window to popup before running inspect_items.py
        sleep(Duration::from_millis(300));

        // Click the magnifying glasses on top of the items
        let output = Command::new("python")
            .arg("python_helpers/inspect_items.py")
            .output()
            .expect("Failed to execute command");

        // Convert the output bytes to a string
        let output_str = str::from_utf8(&output.stdout).unwrap().trim();

        println!("coords: {}", output_str);
        // Split the string on newlines to get the list of coordinates
        let coords: Vec<&str> = output_str.split('\n').collect();

        // Now, coords contains each of the coordinates
        for coord_str in coords.iter() {
            let mut rng = rand::thread_rng();

            if *coord_str == "Could not detect" || *coord_str == "" {
                println!("Counld not find item");
                // Moving away from items for obj detection purposes.
                match enigo_functions::move_to_location_fast(
                    &mut enigo,
                    rng.gen_range(25..50),
                    rng.gen_range(200..300),
                    true,
                ) {
                    Ok(_) => println!("Successfully moved to this location!"),
                    Err(err) => println!("Got error while trying to move cursor: {:?}", err),
                }

                // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
                enigo.mouse_click(MouseButton::Left);
                continue;
            }
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

                match enigo_functions::click_buton_direct(
                    &mut enigo,
                    middle_point_x,
                    middle_point_y,
                    true,
                    false,
                    0,
                    0,
                ) {
                    Ok(_) => println!("Successfully clicked button!"),
                    Err(err) => println!("Got error while trying to click button: {:?}", err),
                }
            }
        }

        // Loop through the items in the trader struct for this trader and use obj detection to check if the item is present
        // If item is present then add it to list.

        let mut rng = rand::thread_rng();

        // Moving away from items for obj detection purposes.
        match enigo_functions::move_to_location_fast(
            &mut enigo,
            rng.gen_range(25..50),
            rng.gen_range(200..300),
            true,
        ) {
            Ok(_) => println!("Successfully moved to this location!"),
            Err(err) => println!("Got error while trying to move cursor: {:?}", err),
        }

        // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
        enigo.mouse_click(MouseButton::Left);

        // Download 1 image set into temp_images folder at a time and check for a match
        let info_vec = &trader.unwrap().info_images_not_traded;
        let item_vec = &trader.unwrap().item_images_not_traded;

        // For each image pair. Download the pair and if there is a matching pair in the trading window, add it to list in memory.
        // After trading successfully, change status to "in escrow" for the traded items in the database.
        let mut trading_window_items = Vec::new();

        for item in item_vec.iter() {
            match download_image(&item, "temp_images/item/image.png") {
                Ok(_) => println!("Successfully downloaded item image"),
                Err(err) => {
                    println!("Could not download image. Error \n{}", err);
                    return_to_lobby();
                    return Err(String::from("Could not download image"));
                }
            }

            let output = Command::new("python")
                .arg("python_helpers/multi_obj_detection_narrow.py")
                .arg("temp_images/item/image.png")
                .arg("SC")
                .arg("F")
                .arg("G")
                .arg("CR")
                .output()
                .expect("Failed to execute command");

            // Convert the output bytes to a string
            let output_str = str::from_utf8(&output.stdout).unwrap().trim();

            println!("detected items: \n{}", output_str);

            // Split the string on newlines to get the list of coordinates
            let coords: Vec<&str> = output_str.split('\n').collect();

            // Now, coords contains each of the coordinates
            for coord_str in coords.iter() {
                if *coord_str == "Could not detect" || *coord_str == "" {
                    println!("Counld not find item");
                    // Moving away from items for obj detection purposes.
                    match enigo_functions::move_to_location_fast(
                        &mut enigo,
                        rng.gen_range(25..50),
                        rng.gen_range(200..300),
                        true,
                    ) {
                        Ok(_) => println!("Successfully moved to this location!"),
                        Err(err) => println!("Got error while trying to move cursor: {:?}", err),
                    }

                    // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
                    enigo.mouse_click(MouseButton::Left);
                    continue;
                }
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
                        false,
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
                                return_to_lobby();
                                return Err(String::from("Could not download image"));
                            }
                        }

                        let output = Command::new("python")
                            .arg("python_helpers/obj_detection.py")
                            .arg("temp_images/info/image.png")
                            .arg("SF")
                            .arg("CR")
                            .output();

                        match output {
                            Ok(out) => {
                                let output_str = str::from_utf8(&out.stdout).unwrap().trim();

                                // Split the string on newlines to get the list of coordinates
                                let coords: Vec<&str> = output_str.split('\n').collect();

                                // Now, coords contains each of the coordinates
                                for coord_str in coords.iter() {
                                    if *coord_str == "Could not detect" || *coord_str == "" {
                                        println!("Could not find match");
                                        // Moving away from items for obj detection purposes.
                                        match enigo_functions::move_to_location_fast(
                                            &mut enigo,
                                            rng.gen_range(25..50),
                                            rng.gen_range(200..300),
                                            true,
                                        ) {
                                            Ok(_) => {
                                                println!("Successfully moved to this location!")
                                            }
                                            Err(err) => println!(
                                                "Got error while trying to move cursor: {:?}",
                                                err
                                            ),
                                        }

                                        // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
                                        enigo.mouse_click(MouseButton::Left);
                                        continue;
                                    }
                                    println!("Found match!");
                                    trading_window_items.push((info_image, item));
                                }
                            }
                            Err(_) => println!("No match. Checking next..."),
                        }
                    }
                }
            }
        }

        // Make copy to use for later
        let trading_window_items_clone = trading_window_items.clone();

        // Moving away from items for obj detection purposes.
        match enigo_functions::move_to_location_fast(
            &mut enigo,
            rng.gen_range(25..50),
            rng.gen_range(200..300),
            true,
        ) {
            Ok(_) => println!("Successfully moved to this location!"),
            Err(err) => println!("Got error while trying to move cursor: {:?}", err),
        }

        // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
        enigo.mouse_click(MouseButton::Left);

        // After checking all the items check the gold amount
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
                println!("Trader accepted the trade!")
            }
            Err(_) => {
                println!("User did not accept trade.");
                // GO TO LOBBY
                return_to_lobby();
                return Err(String::from("User did not accept trade"));
            }
        }

        // Get the amount of gold in the trade
        let output = Command::new("python")
            .arg("python_helpers/total_gold.py")
            .output();

        // Match the output of the gold detector and assigns the amount of gold put in by the trader to the gold variable
        let gold: i32 = match output {
            Ok(out) => {
                let output_str = str::from_utf8(&out.stdout).unwrap().trim();
                if output_str == "No text detected" || output_str == "0" {
                    0
                } else {
                    output_str.parse().unwrap()
                }
            }
            Err(_) => 0,
        };

        let mut result_of_status;

        // If this value is not initialized below, then there is nothing in the trading_window_items_clone which there should be.
        result_of_status = Err(String::from("Something went wrong"));

        if gold > 0 {
            // Add the gold to the trader1_gold_traded or trader2_gold_traded
            let add_gold_result = database_functions::add_gold_to_trader(
                &trader.unwrap().discord_channel_id,
                &trader.unwrap().discord_id,
                gold,
            );

            match add_gold_result {
                Ok(_) => result_of_status = Ok(String::from("Added gold")),
                Err(_) => result_of_status = Err(String::from("Could not add gold")),
            }
        }

        for pair in trading_window_items_clone.iter() {
            match database_functions::set_item_status_by_urls(pair.1, pair.0, "in escrow") {
                Ok(_) => {
                    println!("Updated item status!");
                    result_of_status = Ok(String::from("Success"));
                }
                Err(err) => {
                    println!("Error updating item status. Error: \n{}", err);
                    result_of_status = Err(String::from("Error during trade"))
                }
            }
        }

        // If the result is "Success" then accept the trade, else dont.
        match result_of_status {
            Ok(_) => {
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
                    let mut rng = rand::thread_rng();

                    if *coord_str == "Could not detect" || *coord_str == "" {
                        println!("Counld not find item");
                        // Moving away from items for obj detection purposes.
                        match enigo_functions::move_to_location_fast(
                            &mut enigo,
                            rng.gen_range(25..50),
                            rng.gen_range(200..300),
                            true,
                        ) {
                            Ok(_) => println!("Successfully moved to this location!"),
                            Err(err) => {
                                println!("Got error while trying to move cursor: {:?}", err)
                            }
                        }

                        // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
                        enigo.mouse_click(MouseButton::Left);
                        continue;
                    }
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
                        match enigo_functions::move_to_location_fast(
                            &mut enigo,
                            middle_point_x,
                            middle_point_y,
                            true,
                        ) {
                            Ok(_) => println!("Successfully clicked button!"),
                            Err(err) => {
                                println!("Got error while trying to click button: {:?}", err)
                            }
                        }

                        enigo.mouse_click(MouseButton::Left);
                    }
                }
                return_to_lobby();
                return Ok(String::from("Trade successful"));
            }
            Err(err) => {
                println!("Returning to lobby");
                return_to_lobby();
                return Err(err);
            }
        }
    });
    match result_catch_panic {
        Ok(ok_result) => {
            match ok_result {
                Ok(message) => return Ok(message),
                Err(err) => {
                    return_to_lobby();
                    // Handle the string error here, maybe log it or convert it to your error type
                    return Err(err);
                }
            }
        }
        Err(panic_error) => {
            // This branch handles the case where panic::catch_unwind caught a panic
            // You can log the panic information, perform cleanup, or return an appropriate error
            // Convert panic_error (Box<dyn Any + Send>) to a suitable error type if needed
            return_to_lobby();
            // Return a generic error message
            return Err(format!(
                "Got error while trying to unwrap panic. Error: {:?}",
                panic_error
            ));
        }
    }
}

// Collect items function
pub fn claim_items(
    enigo: Arc<Mutex<Enigo>>,
    bot_info: Arc<Mutex<TradeBotInfo>>,
    in_game_id: &str,
    traders_container: Arc<Mutex<TradersContainer>>,
) -> Result<String, String> {
    let result_catch_panic = panic::catch_unwind(|| {
        let mut enigo = enigo.lock().unwrap();

        let info = bot_info.lock().unwrap();

        // If the bot is not ready then it will run the open game function
        // If the bot is starting then it will wait for the bot to be ready
        // If the bot is ready then it will continue as normal
        'wait_loop: loop {
            let bot_info_clone = bot_info.clone();
            match info.ready {
                ReadyState::False => {
                    tokio::spawn(async move {
                        open_game_go_to_lobby(bot_info_clone, true).await;
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
        let other_trader =
            traders.get_other_trader_in_channel(&trader_discord_id, &trader_channel_id);

        let items_escrow_count = database_functions::items_in_escrow_count(other_trader.unwrap());

        // If there are no items in escrow then just return.
        match items_escrow_count {
            Ok(count) => {
                if count <= 0 {
                    println!("No more items left in escrow.");
                    return Err(String::from("No items left in escrow"));
                }
            }
            Err(err) => {
                println!(
                    "Got error while counting number of items in escrow. Error:\n{}",
                    err
                );
                return Err(String::from("No items left in escrow"));
            }
        }
        match send_trade_request(in_game_id) {
            Ok(_) => println!("Player accepted trade request"),
            Err(_) => {
                println!("Player declined request. Going back to lobby.");
                return_to_lobby();
                return Err(String::from("Player declined trade request"));
            }
        }

        // Now we are in the trading window
        // It should find matches in both the inventory and the stash and add them to the trading window.

        // These 2 vectors store the traders items. It loops through these and find pairs and adds them to the trade.
        let info_vec = &other_trader.unwrap().info_images_escrow;
        let item_vec = &other_trader.unwrap().item_images_escrow;

        // Store the items that made it through in this vector
        // Then when the trade is done loop through the list and change their status from "in escrow" to "traded"
        // (Info, item)
        let mut in_window_items = Vec::new();

        // For each image pair. Download the pair and if there is a matching pair in the stash or inventory, add it to the trading window.
        // Max items that you can have per trade.
        let mut item_limit = 25;
        'add_items: for item in item_vec.iter() {
            if item_limit <= 0 {
                println!("Reached item limit!");
                break 'add_items;
            }
            match download_image(&item, "temp_images/item/image.png") {
                Ok(_) => println!("Successfully downloaded item image"),
                Err(err) => {
                    println!("Could not download image. Error \n{}", err);
                    return_to_lobby();
                    return Err(String::from("Could not download image"));
                }
            }
            //sleep(Duration::from_secs(1));
            println!("Test1");
            // Convert the output bytes to a string
            let output_str = {
                let output = Command::new("python")
                    .arg("python_helpers/multi_obj_detection_inv_stash.py")
                    .arg("temp_images/item/image.png")
                    .arg("SC")
                    .arg("F")
                    .arg("CR")
                    .output()
                    .expect("Failed to execute command");
                println!("Coords: {:?}", output);
                str::from_utf8(&output.stdout).unwrap().trim().to_string()
            };
            println!("Coords: {}", output_str);

            println!("Test2");
            // If it could not detect any items in the inventory then go to stash and try again
            let output_str = if output_str == "Could not detect" {
                let output_stash = Command::new("python")
                    .arg("python_helpers/obj_detection.py")
                    .arg("images/stash.png")
                    .arg("F")
                    .arg("C")
                    .output()
                    .expect("Failed to execute command");

                println!("Test3");
                match enigo_functions::click_buton(&mut enigo, output_stash, true, 0, 0) {
                    Ok(_) => println!("Successfully clicked button!"),
                    Err(err) => println!("Got error while trying to click button: {:?}", err),
                }

                println!("Test4");
                let output_retry = Command::new("python")
                    .arg("python_helpers/multi_obj_detection_inv_stash.py")
                    .arg("temp_images/item/image.png")
                    .arg("SC")
                    .arg("F")
                    .arg("CR")
                    .output()
                    .expect("Failed to execute command");

                println!("Test5");
                // Convert the output bytes to a string
                str::from_utf8(&output_retry.stdout)
                    .unwrap()
                    .trim()
                    .to_string()
            } else {
                println!("Test6");
                output_str
            };

            println!("Test7");
            if output_str == "Could not detect" {
                println!("Test8");
                break 'add_items;
            }

            println!("Test9");
            // Split the string on newlines to get the list of coordinates
            let coords: Vec<&str> = output_str.split('\n').collect();

            println!("Test10");
            // Now, coords contains each of the coordinates
            for coord_str in coords.iter() {
                let mut rng = rand::thread_rng();

                if *coord_str == "Could not detect" || *coord_str == "" {
                    println!("Counld not find item");
                    // Moving away from items for obj detection purposes.
                    match enigo_functions::move_to_location_fast(
                        &mut enigo,
                        rng.gen_range(25..50),
                        rng.gen_range(200..300),
                        true,
                    ) {
                        Ok(_) => println!("Successfully moved to this location!"),
                        Err(err) => println!("Got error while trying to move cursor: {:?}", err),
                    }

                    // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
                    enigo.mouse_click(MouseButton::Left);
                    continue;
                }
                let coord: Vec<i32> = coord_str
                    .split_whitespace()
                    .map(|s| s.parse().expect("Failed to parse coordinate"))
                    .collect();

                println!("Test11");
                println!("Coords: {:?}", coords);
                println!("Coord: {:?}", coord);
                println!("Coord str: {}", coord_str);
                if coord.len() == 4 {
                    println!("Test12");
                    let (x1, y1, x2, y2) = (coord[0], coord[1], coord[2], coord[3]);

                    let mut rng = rand::thread_rng();

                    // Salt the pixels so that it does not click the same pixel every time.
                    let salt = rng.gen_range(-9..9);

                    // Gets the middle of the detected play button and clicks it
                    let middle_point_x = ((x2 - x1) / 2) + x1 + salt;
                    let middle_point_y = ((y2 - y1) / 2) + y1 + salt;

                    println!("Test13");
                    match enigo_functions::move_to_location_fast(
                        &mut enigo,
                        middle_point_x,
                        middle_point_y,
                        false,
                    ) {
                        Ok(_) => println!("Successfully moved to this location!"),
                        Err(err) => println!("Got error while trying to move cursor: {:?}", err),
                    }

                    println!("Test14");
                    // Tries to match every info image with the item and if there is a match then it will add it to the temporary vector variable.
                    for info_image in info_vec.iter() {
                        match download_image(info_image, "temp_images/info/image.png") {
                            Ok(_) => println!("Successfully downloaded info image"),
                            Err(err) => {
                                println!("Could not download image. Error \n{}", err);
                                return_to_lobby();
                                return Err(String::from("Player declined request"));
                            }
                        }

                        println!("Test15");
                        // SHOULD USE A VERSION OF OBJ DETECTION WITH A FASTER TIMEOUT. So that it wont wait for 4 minutes of there is no match
                        let output = Command::new("python")
                            .arg("python_helpers/obj_detection.py")
                            .arg("temp_images/info/image.png")
                            .arg("SF")
                            .arg("CR")
                            //.arg("C")
                            .output();

                        println!("Test16");
                        let output_unwrapped = output.unwrap(); // Bind `Output` to a variable to extend its lifetime
                        let output_str = str::from_utf8(&output_unwrapped.stdout).unwrap().trim();

                        println!("Test17");
                        if output_str != "Could not detect" {
                            println!("Found match!");
                            enigo.mouse_click(MouseButton::Left);
                            in_window_items.push((info_image, item));
                            item_limit += -1;
                            println!("Test18");
                        } else {
                            println!("Test19");
                            println!("No match. Checking next...");
                        }
                    }
                }
            }
        }

        println!("Test20");
        // Click checkbox to get into the confirmation trading window.
        let output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/trade_checkbox.png")
            .output()
            .expect("Failed to execute command");

        match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
            Ok(_) => println!("Successfully clicked button!"),
            Err(err) => println!("Got error while trying to click button: {:?}", err),
        }
        println!("Test21");

        // Check that the trader also has checked the checkbox and that we are now in the trading phase 2/2
        let output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/second_phase_check.png")
            .output();

        match output {
            Ok(_) => {
                println!("Trader accepted the trade!")
            }
            Err(_) => {
                println!("User did not accept trade.");
                // GO TO LOBBY
                return_to_lobby();
                return Err(String::from("User did not accept trade"));
            }
        }
        // Now the bot is in the double check trade window box.
        // Click the magnifying glasses on top of the items, incase the trader put anything in there
        let output = Command::new("python")
            .arg("python_helpers/inspect_items.py")
            .output()
            .expect("Failed to execute command");

        // Convert the output bytes to a string
        let output_str = str::from_utf8(&output.stdout).unwrap().trim();

        // Split the string on newlines to get the list of coordinates
        let coords: Vec<&str> = output_str.split('\n').collect();

        println!("Test22");
        // Now, coords contains each of the coordinates
        for coord_str in coords.iter() {
            let mut rng = rand::thread_rng();

            if *coord_str == "Could not detect" || *coord_str == "" {
                println!("Counld not find item");
                // Moving away from items for obj detection purposes.
                match enigo_functions::move_to_location_fast(
                    &mut enigo,
                    rng.gen_range(25..50),
                    rng.gen_range(200..300),
                    true,
                ) {
                    Ok(_) => println!("Successfully moved to this location!"),
                    Err(err) => println!("Got error while trying to move cursor: {:?}", err),
                }

                // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
                enigo.mouse_click(MouseButton::Left);
                continue;
            }
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

                println!("Test23");
                match enigo_functions::click_buton_direct(
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

        println!("Test24");
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
                    return_to_lobby();
                    return Err(String::from("Could not download image"));
                }
            }
            println!("Test25");

            // Using narrow version of multi obj detection.
            // Because the inventory/stash is still visable on this screen so the screenshot that the bot takes needs to be narrowed to only the trading window.

            // Handling output and avoiding temporary value drop issue
            let output_result = Command::new("python")
                .arg("python_helpers/multi_obj_detection_narrow.py")
                .arg("temp_images/item/image.png")
                .arg("SC")
                .arg("F")
                .arg("G")
                .arg("CR")
                .output();

            println!("Test26");
            match output_result {
                Ok(output) => {
                    println!("Test27");
                    let output_bytes = output.stdout;
                    let output_str = str::from_utf8(&output_bytes).unwrap().trim();
                    let coords: Vec<&str> = output_str.split('\n').collect();

                    // Now, coords contains each of the coordinates
                    for coord_str in coords.iter() {
                        let mut rng = rand::thread_rng();

                        if *coord_str == "Could not detect" || *coord_str == "" {
                            println!("Counld not find item");
                            // Moving away from items for obj detection purposes.
                            match enigo_functions::move_to_location_fast(
                                &mut enigo,
                                rng.gen_range(25..50),
                                rng.gen_range(200..300),
                                true,
                            ) {
                                Ok(_) => println!("Successfully moved to this location!"),
                                Err(err) => {
                                    println!("Got error while trying to move cursor: {:?}", err)
                                }
                            }

                            // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
                            enigo.mouse_click(MouseButton::Left);
                            continue;
                        }
                        let coord: Vec<i32> = coord_str
                            .split_whitespace()
                            .map(|s| s.parse().expect("Failed to parse coordinate"))
                            .collect();

                        println!("Test28");
                        if coord.len() == 4 {
                            println!("Test28");
                            let (x1, y1, x2, y2) = (coord[0], coord[1], coord[2], coord[3]);

                            let mut rng = rand::thread_rng();

                            // Salt the pixels so that it does not click the same pixel every time.
                            let salt = rng.gen_range(-9..9);

                            // Gets the middle of the detected play button and clicks it
                            let middle_point_x = ((x2 - x1) / 2) + x1 + salt;
                            let middle_point_y = ((y2 - y1) / 2) + y1 + salt;

                            println!("Test29");
                            match enigo_functions::move_to_location_fast(
                                &mut enigo,
                                middle_point_x,
                                middle_point_y,
                                true,
                            ) {
                                Ok(_) => println!("Successfully moved to this location!"),
                                Err(err) => {
                                    println!("Got error while trying to move cursor: {:?}", err)
                                }
                            }

                            println!("Test30");
                            // Tries to match every info image with the item and if there is a match then it will add it to the temporary vector variable.
                            for info_image in info_vec.iter() {
                                match download_image(info_image, "temp_images/info/image.png") {
                                    Ok(_) => println!("Successfully downloaded info image"),
                                    Err(err) => {
                                        println!("Could not download image. Error \n{}", err);
                                        return_to_lobby();
                                        return Err(String::from("Could not download image"));
                                    }
                                }

                                println!("Test31");
                                let output = Command::new("python")
                                    .arg("python_helpers/obj_detection.py")
                                    .arg("temp_images/info/image.png")
                                    .arg("SF")
                                    .arg("CR")
                                    .output();

                                println!("Test32");
                                match output {
                                    Ok(_) => {
                                        println!("Found match!");
                                        // Moving away from items for obj detection purposes.
                                        match enigo_functions::move_to_location_fast(
                                            &mut enigo,
                                            rng.gen_range(25..50),
                                            rng.gen_range(200..300),
                                            true,
                                        ) {
                                            Ok(_) => {
                                                println!("Successfully moved to this location!")
                                            }
                                            Err(err) => println!(
                                                "Got error while trying to move cursor: {:?}",
                                                err
                                            ),
                                        }

                                        // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
                                        enigo.mouse_click(MouseButton::Left);
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
                    println!("Test33");
                    println!("Could not find item. Cancelling trade and going to lobby..");
                    // GO TO LOBBY
                    return_to_lobby();
                    return Err(String::from("Could not find item"));
                }
            }
        }

        println!("Test34");
        // Check if trading_window_items is empty
        if in_window_items.is_empty() {
            println!("Test35");
            println!("No matches where found! Going back to lobby");
            return_to_lobby();
            return Err(String::from("No items found"));
        }
        // If the in_window_items is not emtpy then change the status of those images from "in escrow" to "traded"
        else {
            println!("Test36");
            // The bot ensures that the trade went through by making sure that it is the last link in the trade.
            // The bot waits for the trader to accept the trade by clicking the checkmark before the bot itself does.
            // Right as the trader clicks the button, the bot does as well, completing the trade for centain.
            // SHOULD USE A VERSION OF OBJ DETECTION WITH A FASTER TIMEOUT. So that it won't wait for 4 minutes if there is no match
            let output = Command::new("python")
                .arg("python_helpers/obj_detection.py")
                .arg("images/trader_ready.png")
                .output();

            println!("Test37");
            match output {
                Ok(_) => {
                    println!("User accepted trade!");
                    // Click the checkbox fast so that the other trader does not have time to decline in order to try to trick the bot.
                    let output = Command::new("python")
                        .arg("python_helpers/obj_detection.py")
                        .arg("images/trade_checkbox.png")
                        .output()
                        .expect("Failed to execute command");

                    println!("Test38");
                    // Convert the output into 4 coordinates and get the middle point of those.
                    // Then use the move_to_location_fast function to quickly move to the checkbox and click it
                    // Convert the output bytes to a string
                    let output_str = str::from_utf8(&output.stdout).unwrap().trim();

                    // Split the string on newlines to get the list of coordinates
                    let coords: Vec<&str> = output_str.split('\n').collect();

                    println!("Test39");
                    // Now, coords contains each of the coordinates
                    for coord_str in coords.iter() {
                        let mut rng = rand::thread_rng();

                        if *coord_str == "Could not detect" || *coord_str == "" {
                            println!("Counld not find item");
                            // Moving away from items for obj detection purposes.
                            match enigo_functions::move_to_location_fast(
                                &mut enigo,
                                rng.gen_range(25..50),
                                rng.gen_range(200..300),
                                true,
                            ) {
                                Ok(_) => println!("Successfully moved to this location!"),
                                Err(err) => {
                                    println!("Got error while trying to move cursor: {:?}", err)
                                }
                            }

                            // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
                            enigo.mouse_click(MouseButton::Left);
                            continue;
                        }
                        let coord: Vec<i32> = coord_str
                            .split_whitespace()
                            .map(|s| s.parse().expect("Failed to parse coordinate"))
                            .collect();

                        println!("Test40");
                        if coord.len() == 4 {
                            let (x1, y1, x2, y2) = (coord[0], coord[1], coord[2], coord[3]);

                            let mut rng = rand::thread_rng();

                            // Salt the pixels so that it does not click the same pixel every time.
                            let salt = rng.gen_range(-9..9);

                            // Gets the middle of the detected play button and clicks it
                            let middle_point_x = ((x2 - x1) / 2) + x1 + salt;
                            let middle_point_y = ((y2 - y1) / 2) + y1 + salt;

                            println!("Test41");
                            // Now move to the middlepoint
                            match enigo_functions::move_to_location_fast(
                                &mut enigo,
                                middle_point_x,
                                middle_point_y,
                                true,
                            ) {
                                Ok(_) => println!("Successfully clicked button!"),
                                Err(err) => {
                                    println!("Got error while trying to click button: {:?}", err)
                                }
                            }

                            enigo.mouse_click(MouseButton::Left);
                        }
                    }
                }
                // Might not work...
                Err(_) => {
                    println!("Test42");
                    println!("User did not accept trade.");
                    // GO TO LOBBY
                    return_to_lobby();
                    return Err(String::from("User did not accept trade"));
                }
            }

            println!("Test43");
            println!("Changing the items statuses from 'in escrow' to 'traded'!");
            for (info_url, item_url) in in_window_items {
                match database_functions::set_item_status_by_urls(item_url, info_url, "traded") {
                    Ok(_) => println!("Changed the item status for 1 item!"),
                    Err(err) => println!("Got error while changing item status. Error: \n{}", err),
                }
            }
        }
        println!("Test44");

        return_to_lobby();
        Ok(String::from("Trade successful"))
    });
    match result_catch_panic {
        Ok(ok_result) => {
            match ok_result {
                Ok(message) => return Ok(message),
                Err(err) => {
                    return_to_lobby();
                    // Handle the string error here, maybe log it or convert it to your error type
                    return Err(err);
                }
            }
        }
        Err(panic_error) => {
            // This branch handles the case where panic::catch_unwind caught a panic
            // You can log the panic information, perform cleanup, or return an appropriate error
            // Convert panic_error (Box<dyn Any + Send>) to a suitable error type if needed
            return_to_lobby();
            // Return a generic error message
            return Err(format!(
                "Got error while trying to unwrap panic. Error: {:?}",
                panic_error
            ));
        }
    }
}

pub fn claim_gold(
    enigo: Arc<Mutex<Enigo>>,
    bot_info: Arc<Mutex<TradeBotInfo>>,
    in_game_id: &str,
    traders_container: Arc<Mutex<TradersContainer>>,
) -> Result<String, String> {
    let result_catch_panic = panic::catch_unwind(|| {
        let mut enigo = enigo.lock().unwrap();

        let info = bot_info.lock().unwrap();

        // If the bot is not ready then it will run the open game function
        // If the bot is starting then it will wait for the bot to be ready
        // If the bot is ready then it will continue as normal
        'wait_loop: loop {
            let bot_info_clone = bot_info.clone();
            match info.ready {
                ReadyState::False => {
                    tokio::spawn(async move {
                        open_game_go_to_lobby(bot_info_clone, true).await;
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
        let other_trader =
            traders.get_other_trader_in_channel(&trader_discord_id, &trader_channel_id);

        let other_trader_gold = other_trader.unwrap().gold;

        if other_trader_gold < 30 {
            return Err(String::from(
                "User did not have enough gold left for a trade",
            ));
        }

        // Go into the trading tab and send a trade to the trader. Exact same as before with the gold fee.
        match send_trade_request(trader.in_game_id.as_str()) {
            Ok(_) => println!("Player accepted trade request"),
            Err(_) => {
                println!("Player declined request. Going back to lobby.");
                return_to_lobby();
                return Err(String::from("Player declined trade request"));
            }
        }

        // Get the amount of 50g and 35g pouches from both the inventory and the stash, while in the trading window.
        let output_50g_inv = Command::new("python")
            .arg("python_helpers/multi_obj_detection_inv_stash.py")
            .arg("images/50_gold_pouch.png")
            .arg("F")
            .output()
            .expect("Failed to execute command");

        sleep(Duration::from_millis(500));

        let output_35g_inv = Command::new("python")
            .arg("python_helpers/multi_obj_detection_inv_stash.py")
            .arg("images/35_gold_pouch.png")
            .arg("F")
            .output()
            .expect("Failed to execute command");

        // Convert the output bytes to a String
        let coords_50g_pouch_inv = str::from_utf8(&output_50g_inv.stdout).unwrap().trim();
        let coords_35g_pouch_inv = str::from_utf8(&output_35g_inv.stdout).unwrap().trim();

        println!("coord 35: {}", coords_35g_pouch_inv);
        println!("coord 35: {}", coords_50g_pouch_inv);
        println!("coord test");

        // Split the string by lines and count them
        let mut pouch_count_50g_inv = 0;
        if coords_50g_pouch_inv != "Could not detect" {
            pouch_count_50g_inv = coords_50g_pouch_inv.lines().count() as i32;
        }

        let mut pouch_count_35g_inv = 0;
        if coords_35g_pouch_inv != "Could not detect" {
            pouch_count_35g_inv = coords_35g_pouch_inv.lines().count() as i32;
        }

        // Now go to the stash and count those as well
        let stash_output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/stash.png")
            .arg("F")
            .arg("C")
            .output()
            .expect("Failed to execute command");

        match enigo_functions::click_buton(&mut enigo, stash_output, true, 0, 0) {
            Ok(_) => println!("Successfully clicked button!"),
            Err(err) => println!("Got error while trying to click button: {:?}", err),
        }

        sleep(Duration::from_millis(500));

        let output_50g_stash = Command::new("python")
            .arg("python_helpers/multi_obj_detection_inv_stash.py")
            .arg("images/50_gold_pouch.png")
            .arg("F")
            .output()
            .expect("Failed to execute command");

        sleep(Duration::from_millis(500));

        let output_35g_stash = Command::new("python")
            .arg("python_helpers/multi_obj_detection_inv_stash.py")
            .arg("images/35_gold_pouch.png")
            .arg("F")
            .output()
            .expect("Failed to execute command");

        // Convert the output bytes to a String
        let coords_50g_pouch_stash = str::from_utf8(&output_50g_stash.stdout).unwrap().trim();
        let coords_35g_pouch_stash = str::from_utf8(&output_35g_stash.stdout).unwrap().trim();

        // Split the string by lines and count them
        let mut pouch_count_50g_stash = 0;
        if coords_50g_pouch_stash != "Could not detect" {
            pouch_count_50g_stash = coords_50g_pouch_stash.lines().count() as i32;
        }

        let mut pouch_count_35g_stash = 0;
        if coords_35g_pouch_stash != "Could not detect" {
            pouch_count_35g_stash = coords_35g_pouch_stash.lines().count() as i32;
        }

        // Going back to inventory screen
        let stash_output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/inventory.png")
            .arg("F")
            .arg("C")
            .output()
            .expect("Failed to execute command");

        match enigo_functions::click_buton(&mut enigo, stash_output, true, 0, 0) {
            Ok(_) => println!("Successfully clicked button!"),
            Err(err) => println!("Got error while trying to click button: {:?}", err),
        }

        // Count total amount of gold and check if there is enough to complete the transaction
        let total_gold = ((pouch_count_50g_inv + pouch_count_50g_stash) * 50)
            + ((pouch_count_35g_inv + pouch_count_35g_stash) * 35);
        if total_gold < (other_trader_gold - 30) {
            return_to_lobby();
            return Err(String::from("Not enough gold"));
        }

        // Now the bot has counted the total amount of 50g and 35g pouches available.
        // We also have all the coordiantes for the different pouches in both the inventory and stash.
        // And we know that the bot has sufficient funds
        // Run the pouch calculator algorithim to calculate how many and where those pouches should come from

        println!(
            "{} {} {} {}",
            pouch_count_50g_inv, pouch_count_50g_stash, pouch_count_35g_inv, pouch_count_35g_stash
        );

        let (inv_50, stash_50, inv_35, stash_35) = calculate_pouches(
            other_trader_gold,
            pouch_count_50g_inv,
            pouch_count_50g_stash,
            pouch_count_35g_inv,
            pouch_count_35g_stash,
        );

        let mut clicked_pouches = 0;
        let max_window_pouches = 25;

        // If there are any pouches present in the inventory then click them
        if inv_50 > 0 || inv_35 > 0 {
            // Loop through the 50's in the inventory and click on them
            clicked_pouches = click_pouches(
                coords_50g_pouch_inv,
                inv_50,
                clicked_pouches,
                max_window_pouches,
            );

            // Loop through the 35's in the inventory and click on them
            clicked_pouches = click_pouches(
                coords_35g_pouch_inv,
                inv_35,
                clicked_pouches,
                max_window_pouches,
            );
        }

        // If there are any pouches present in the stash then click them
        // Going into stash and clicking on those as well
        if stash_50 > 0 || stash_35 > 0 {
            let stash_output = Command::new("python")
                .arg("python_helpers/obj_detection.py")
                .arg("images/stash.png")
                .arg("F")
                .arg("C")
                .output()
                .expect("Failed to execute command");

            match enigo_functions::click_buton(&mut enigo, stash_output, true, 0, 0) {
                Ok(_) => println!("Successfully clicked button!"),
                Err(err) => println!("Got error while trying to click button: {:?}", err),
            }

            // Loop through the 50's in the stash and click on them
            clicked_pouches = click_pouches(
                coords_50g_pouch_stash,
                stash_50,
                clicked_pouches,
                max_window_pouches,
            );

            // Loop through the 35's in the stash and click on them
            click_pouches(
                coords_35g_pouch_stash,
                stash_35,
                clicked_pouches,
                max_window_pouches,
            );
        }

        // Now all the gold is in the trading window.
        // Wait for the trader to be ready and then accept the trade
        let output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/trader_ready.png")
            .output();

        match output {
            Ok(_) => println!("User did accept the trade"),
            Err(_) => {
                println!("User did not accept trade");
                return_to_lobby();
                return Err(String::from("User did not accept trade"));
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

        // Wait for trading window to popup before running inspect_items.py
        sleep(Duration::from_millis(300));

        // Click the magnifying glasses on top of the items
        let output = Command::new("python")
            .arg("python_helpers/inspect_items.py")
            .output()
            .expect("Failed to execute command");

        // Convert the output bytes to a string
        let output_str = str::from_utf8(&output.stdout).unwrap().trim();

        println!("coords: {}", output_str);
        // Split the string on newlines to get the list of coordinates
        let coords: Vec<&str> = output_str.split('\n').collect();

        // Now, coords contains each of the coordinates
        for coord_str in coords.iter() {
            let mut rng = rand::thread_rng();

            if *coord_str == "Could not detect" || *coord_str == "" {
                println!("Counld not find item");
                // Moving away from items for obj detection purposes.
                match enigo_functions::move_to_location_fast(
                    &mut enigo,
                    rng.gen_range(25..50),
                    rng.gen_range(200..300),
                    true,
                ) {
                    Ok(_) => println!("Successfully moved to this location!"),
                    Err(err) => println!("Got error while trying to move cursor: {:?}", err),
                }

                // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
                enigo.mouse_click(MouseButton::Left);
                continue;
            }
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

                match enigo_functions::click_buton_direct(
                    &mut enigo,
                    middle_point_x,
                    middle_point_y,
                    true,
                    false,
                    0,
                    0,
                ) {
                    Ok(_) => println!("Successfully clicked button!"),
                    Err(err) => println!("Got error while trying to click button: {:?}", err),
                }
            }
        }

        // Read the total amount of gold from the Total gold section of the trade.
        // Get the amount of gold in the trade
        let output = Command::new("python")
            .arg("python_helpers/total_bot_gold.py")
            .output();

        // Match the output of the gold detector and assigns the amount of gold put in by the trader to the gold variable
        let gold: i32 = match output {
            Ok(out) => {
                let output_str = str::from_utf8(&out.stdout).unwrap().trim();
                if output_str == "No text detected" || output_str == "0" {
                    0
                } else {
                    output_str.parse().unwrap()
                }
            }
            Err(_) => 0,
        };

        // Wait for trader to click the checkbox again before finishing the trade
        let output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/trader_ready.png")
            .output();

        match output {
            Ok(_) => println!("User did accept the trade"),
            Err(_) => {
                println!("User did not accept trade");
                return_to_lobby();
                return Err(String::from("User did not accept trade"));
            }
        }

        // Click the checkbox
        let output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/trade_checkbox.png")
            .output()
            .expect("Failed to execute command");

        // If it went through, add the total amount of gold to the trader_gold_received and return Ok(String::from("Successfully went through")).
        match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
            Ok(_) => {
                println!("Successfully clicked button!");
                // Does not update database
                // This will add the gold just traded to the total received
                match database_functions::add_gold_received_to_trader(
                    &trader.discord_channel_id,
                    &other_trader.unwrap().discord_id,
                    gold,
                ) {
                    Ok(_) => {
                        return_to_lobby();
                        return Ok(String::from("Trade successful"));
                    }
                    Err(err) => {
                        println!("Error adding gold for trader. \nError:\n{}", err);
                        return_to_lobby();
                        return Err(String::from("Error adding gold for trader"));
                    }
                }
            }
            Err(err) => {
                println!("Got error while trying to click button: {:?}", err);
                return_to_lobby();
                return Err(String::from("Error while trying to click button"));
            }
        }
    });
    match result_catch_panic {
        Ok(ok_result) => {
            match ok_result {
                Ok(message) => return Ok(message),
                Err(err) => {
                    return_to_lobby();
                    // Handle the string error here, maybe log it or convert it to your error type
                    return Err(err);
                }
            }
        }
        Err(panic_error) => {
            // This branch handles the case where panic::catch_unwind caught a panic
            // You can log the panic information, perform cleanup, or return an appropriate error
            // Convert panic_error (Box<dyn Any + Send>) to a suitable error type if needed
            return_to_lobby();
            // Return a generic error message
            return Err(format!(
                "Got error while trying to unwrap panic. Error: {:?}",
                panic_error
            ));
        }
    }
}

pub fn return_gold(
    enigo: Arc<Mutex<Enigo>>,
    bot_info: Arc<Mutex<TradeBotInfo>>,
    in_game_id: &str,
    traders_container: Arc<Mutex<TradersContainer>>,
) -> Result<String, String> {
    let result_catch_panic = panic::catch_unwind(|| {
        let mut enigo = enigo.lock().unwrap();

        let info = bot_info.lock().unwrap();

        // If the bot is not ready then it will run the open game function
        // If the bot is starting then it will wait for the bot to be ready
        // If the bot is ready then it will continue as normal
        'wait_loop: loop {
            let bot_info_clone = bot_info.clone();
            match info.ready {
                ReadyState::False => {
                    tokio::spawn(async move {
                        open_game_go_to_lobby(bot_info_clone, true).await;
                    });
                }
                ReadyState::Starting => sleep(Duration::from_secs(2)),
                ReadyState::True => break 'wait_loop,
            }
        }

        // Get the trader with that in-game name
        let traders = traders_container.lock().unwrap();
        let trader = traders.get_trader_by_in_game_id(in_game_id).unwrap();

        let trader_gold = trader.gold;

        if trader_gold < 30 {
            return Err(String::from(
                "User did not have enough gold left for a trade",
            ));
        }

        // Go into the trading tab and send a trade to the trader. Exact same as before with the gold fee.
        match send_trade_request(trader.in_game_id.as_str()) {
            Ok(_) => println!("Player accepted trade request"),
            Err(_) => {
                println!("Player declined request. Going back to lobby.");
                return_to_lobby();
                return Err(String::from("Player declined trade request"));
            }
        }

        // Get the amount of 50g and 35g pouches from both the inventory and the stash, while in the trading window.
        let output_50g_inv = Command::new("python")
            .arg("python_helpers/multi_obj_detection_inv_stash.py")
            .arg("images/50_gold_pouch.png")
            .arg("F")
            .output()
            .expect("Failed to execute command");

        sleep(Duration::from_millis(500));

        let output_35g_inv = Command::new("python")
            .arg("python_helpers/multi_obj_detection_inv_stash.py")
            .arg("images/35_gold_pouch.png")
            .arg("F")
            .output()
            .expect("Failed to execute command");

        // Convert the output bytes to a String
        let coords_50g_pouch_inv = str::from_utf8(&output_50g_inv.stdout).unwrap().trim();
        let coords_35g_pouch_inv = str::from_utf8(&output_35g_inv.stdout).unwrap().trim();

        println!("coord 35: {}", coords_35g_pouch_inv);
        println!("coord 35: {}", coords_50g_pouch_inv);
        println!("coord test");

        // Split the string by lines and count them
        let mut pouch_count_50g_inv = 0;
        if coords_50g_pouch_inv != "Could not detect" {
            pouch_count_50g_inv = coords_50g_pouch_inv.lines().count() as i32;
        }

        let mut pouch_count_35g_inv = 0;
        if coords_35g_pouch_inv != "Could not detect" {
            pouch_count_35g_inv = coords_35g_pouch_inv.lines().count() as i32;
        }

        // Now go to the stash and count those as well
        let stash_output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/stash.png")
            .arg("F")
            .arg("C")
            .output()
            .expect("Failed to execute command");

        match enigo_functions::click_buton(&mut enigo, stash_output, true, 0, 0) {
            Ok(_) => println!("Successfully clicked button!"),
            Err(err) => println!("Got error while trying to click button: {:?}", err),
        }

        sleep(Duration::from_millis(500));

        let output_50g_stash = Command::new("python")
            .arg("python_helpers/multi_obj_detection_inv_stash.py")
            .arg("images/50_gold_pouch.png")
            .arg("F")
            .output()
            .expect("Failed to execute command");

        sleep(Duration::from_millis(500));

        let output_35g_stash = Command::new("python")
            .arg("python_helpers/multi_obj_detection_inv_stash.py")
            .arg("images/35_gold_pouch.png")
            .arg("F")
            .output()
            .expect("Failed to execute command");

        // Convert the output bytes to a String
        let coords_50g_pouch_stash = str::from_utf8(&output_50g_stash.stdout).unwrap().trim();
        let coords_35g_pouch_stash = str::from_utf8(&output_35g_stash.stdout).unwrap().trim();

        // Split the string by lines and count them
        let mut pouch_count_50g_stash = 0;
        if coords_50g_pouch_stash != "Could not detect" {
            pouch_count_50g_stash = coords_50g_pouch_stash.lines().count() as i32;
        }

        let mut pouch_count_35g_stash = 0;
        if coords_35g_pouch_stash != "Could not detect" {
            pouch_count_35g_stash = coords_35g_pouch_stash.lines().count() as i32;
        }

        // Going back to inventory screen
        let stash_output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/inventory.png")
            .arg("F")
            .arg("C")
            .output()
            .expect("Failed to execute command");

        match enigo_functions::click_buton(&mut enigo, stash_output, true, 0, 0) {
            Ok(_) => println!("Successfully clicked button!"),
            Err(err) => println!("Got error while trying to click button: {:?}", err),
        }

        // Count total amount of gold and check if there is enough to complete the transaction
        let total_gold = ((pouch_count_50g_inv + pouch_count_50g_stash) * 50)
            + ((pouch_count_35g_inv + pouch_count_35g_stash) * 35);
        if total_gold < (trader_gold - 30) {
            return_to_lobby();
            return Err(String::from("Not enough gold"));
        }

        // Now the bot has counted the total amount of 50g and 35g pouches available.
        // We also have all the coordiantes for the different pouches in both the inventory and stash.
        // And we know that the bot has sufficient funds
        // Run the pouch calculator algorithim to calculate how many and where those pouches should come from

        println!(
            "{} {} {} {}",
            pouch_count_50g_inv, pouch_count_50g_stash, pouch_count_35g_inv, pouch_count_35g_stash
        );

        let (inv_50, stash_50, inv_35, stash_35) = calculate_pouches(
            trader_gold,
            pouch_count_50g_inv,
            pouch_count_50g_stash,
            pouch_count_35g_inv,
            pouch_count_35g_stash,
        );

        let mut clicked_pouches = 0;
        let max_window_pouches = 25;

        // If there are any pouches present in the inventory then click them
        if inv_50 > 0 || inv_35 > 0 {
            // Loop through the 50's in the inventory and click on them
            clicked_pouches = click_pouches(
                coords_50g_pouch_inv,
                inv_50,
                clicked_pouches,
                max_window_pouches,
            );

            // Loop through the 35's in the inventory and click on them
            clicked_pouches = click_pouches(
                coords_35g_pouch_inv,
                inv_35,
                clicked_pouches,
                max_window_pouches,
            );
        }

        // If there are any pouches present in the stash then click them
        // Going into stash and clicking on those as well
        if stash_50 > 0 || stash_35 > 0 {
            let stash_output = Command::new("python")
                .arg("python_helpers/obj_detection.py")
                .arg("images/stash.png")
                .arg("F")
                .arg("C")
                .output()
                .expect("Failed to execute command");

            match enigo_functions::click_buton(&mut enigo, stash_output, true, 0, 0) {
                Ok(_) => println!("Successfully clicked button!"),
                Err(err) => println!("Got error while trying to click button: {:?}", err),
            }

            // Loop through the 50's in the stash and click on them
            clicked_pouches = click_pouches(
                coords_50g_pouch_stash,
                stash_50,
                clicked_pouches,
                max_window_pouches,
            );

            // Loop through the 35's in the stash and click on them
            click_pouches(
                coords_35g_pouch_stash,
                stash_35,
                clicked_pouches,
                max_window_pouches,
            );
        }

        // Now all the gold is in the trading window.
        // Wait for the trader to be ready and then accept the trade
        let output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/trader_ready.png")
            .output();

        match output {
            Ok(_) => println!("User did accept the trade"),
            Err(_) => {
                println!("User did not accept trade");
                return_to_lobby();
                return Err(String::from("User did not accept trade"));
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

        // Wait for trading window to popup before running inspect_items.py
        sleep(Duration::from_millis(300));

        // Click the magnifying glasses on top of the items
        let output = Command::new("python")
            .arg("python_helpers/inspect_items.py")
            .output()
            .expect("Failed to execute command");

        // Convert the output bytes to a string
        let output_str = str::from_utf8(&output.stdout).unwrap().trim();

        println!("coords: {}", output_str);
        // Split the string on newlines to get the list of coordinates
        let coords: Vec<&str> = output_str.split('\n').collect();

        // Now, coords contains each of the coordinates
        for coord_str in coords.iter() {
            let mut rng = rand::thread_rng();

            if *coord_str == "Could not detect" || *coord_str == "" {
                println!("Counld not find item");
                // Moving away from items for obj detection purposes.
                match enigo_functions::move_to_location_fast(
                    &mut enigo,
                    rng.gen_range(25..50),
                    rng.gen_range(200..300),
                    true,
                ) {
                    Ok(_) => println!("Successfully moved to this location!"),
                    Err(err) => println!("Got error while trying to move cursor: {:?}", err),
                }

                // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
                enigo.mouse_click(MouseButton::Left);
                continue;
            }
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

                match enigo_functions::click_buton_direct(
                    &mut enigo,
                    middle_point_x,
                    middle_point_y,
                    true,
                    false,
                    0,
                    0,
                ) {
                    Ok(_) => println!("Successfully clicked button!"),
                    Err(err) => println!("Got error while trying to click button: {:?}", err),
                }
            }
        }

        // Read the total amount of gold from the Total gold section of the trade.
        // Get the amount of gold in the trade
        let output = Command::new("python")
            .arg("python_helpers/total_bot_gold.py")
            .output();

        // Match the output of the gold detector and assigns the amount of gold put in by the trader to the gold variable
        let gold: i32 = match output {
            Ok(out) => {
                let output_str = str::from_utf8(&out.stdout).unwrap().trim();
                if output_str == "No text detected" || output_str == "0" {
                    0
                } else {
                    output_str.parse().unwrap()
                }
            }
            Err(_) => 0,
        };

        // Wait for trader to click the checkbox again before finishing the trade
        let output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/trader_ready.png")
            .output();

        match output {
            Ok(_) => println!("User did accept the trade"),
            Err(_) => {
                println!("User did not accept trade");
                return_to_lobby();
                return Err(String::from("User did not accept trade"));
            }
        }

        // Click the checkbox
        let output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/trade_checkbox.png")
            .output()
            .expect("Failed to execute command");

        // If it went through, add the total amount of gold to the trader_gold_received and return Ok(String::from("Successfully went through")).
        match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
            Ok(_) => {
                println!("Successfully clicked button!");
                // Does not update database
                // This will add the gold just traded to the total received
                match database_functions::add_gold_received_to_trader(
                    &trader.discord_channel_id,
                    &trader.discord_id,
                    -gold,
                ) {
                    Ok(_) => {
                        return_to_lobby();
                        return Ok(String::from("Trade successful"));
                    }
                    Err(err) => {
                        println!("Error adding gold for trader. \nError:\n{}", err);
                        return_to_lobby();
                        return Err(String::from("Error adding gold for trader"));
                    }
                }
            }
            Err(err) => {
                println!("Got error while trying to click button: {:?}", err);
                return_to_lobby();
                return Err(String::from("Error while trying to click button"));
            }
        }
    });
    match result_catch_panic {
        Ok(ok_result) => {
            match ok_result {
                Ok(message) => return Ok(message),
                Err(err) => {
                    return_to_lobby();
                    // Handle the string error here, maybe log it or convert it to your error type
                    return Err(err);
                }
            }
        }
        Err(panic_error) => {
            // This branch handles the case where panic::catch_unwind caught a panic
            // You can log the panic information, perform cleanup, or return an appropriate error
            // Convert panic_error (Box<dyn Any + Send>) to a suitable error type if needed
            return_to_lobby();
            // Return a generic error message
            return Err(format!(
                "Got error while trying to unwrap panic. Error: {:?}",
                panic_error
            ));
        }
    }
}
pub fn return_items(
    enigo: Arc<Mutex<Enigo>>,
    bot_info: Arc<Mutex<TradeBotInfo>>,
    in_game_id: &str,
    traders_container: Arc<Mutex<TradersContainer>>,
) -> Result<String, String> {
    let result_catch_panic = panic::catch_unwind(|| {
        let mut enigo = enigo.lock().unwrap();

        let info = bot_info.lock().unwrap();

        // If the bot is not ready then it will run the open game function
        // If the bot is starting then it will wait for the bot to be ready
        // If the bot is ready then it will continue as normal
        'wait_loop: loop {
            let bot_info_clone = bot_info.clone();
            match info.ready {
                ReadyState::False => {
                    tokio::spawn(async move {
                        open_game_go_to_lobby(bot_info_clone, true).await;
                    });
                }
                ReadyState::Starting => sleep(Duration::from_secs(2)),
                ReadyState::True => break 'wait_loop,
            }
        }

        // Get the trader with that in-game name
        let traders = traders_container.lock().unwrap();
        let trader = traders.get_trader_by_in_game_id(in_game_id).unwrap();

        let items_escrow_count = database_functions::items_in_escrow_count(trader);

        // If there are no items in escrow then just return.
        match items_escrow_count {
            Ok(count) => {
                if count <= 0 {
                    println!("No more items left in escrow.");
                    return Err(String::from("No items left in escrow"));
                }
            }
            Err(err) => {
                println!(
                    "Got error while counting number of items in escrow. Error:\n{}",
                    err
                );
                return Err(String::from("No items left in escrow"));
            }
        }
        match send_trade_request(in_game_id) {
            Ok(_) => println!("Player accepted trade request"),
            Err(_) => {
                println!("Player declined request. Going back to lobby.");
                return_to_lobby();
                return Err(String::from("Player declined trade request"));
            }
        }

        // Now we are in the trading window
        // It should find matches in both the inventory and the stash and add them to the trading window.

        // These 2 vectors store the traders items. It loops through these and find pairs and adds them to the trade.
        let info_vec = &trader.info_images_escrow;
        let item_vec = &trader.item_images_escrow;

        // Store the items that made it through in this vector
        // Then when the trade is done loop through the list and change their status from "in escrow" to "traded"
        // (Info, item)
        let mut in_window_items = Vec::new();

        // For each image pair. Download the pair and if there is a matching pair in the stash or inventory, add it to the trading window.
        // Max items that you can have per trade.
        let mut item_limit = 25;
        'add_items: for item in item_vec.iter() {
            if item_limit <= 0 {
                println!("Reached item limit!");
                break 'add_items;
            }
            match download_image(&item, "temp_images/item/image.png") {
                Ok(_) => println!("Successfully downloaded item image"),
                Err(err) => {
                    println!("Could not download image. Error \n{}", err);
                    return_to_lobby();
                    return Err(String::from("Could not download image"));
                }
            }
            //sleep(Duration::from_secs(1));
            println!("Test1");
            // Convert the output bytes to a string
            let output_str = {
                let output = Command::new("python")
                    .arg("python_helpers/multi_obj_detection_inv_stash.py")
                    .arg("temp_images/item/image.png")
                    .arg("SC")
                    .arg("F")
                    .arg("CR")
                    .output()
                    .expect("Failed to execute command");
                println!("Coords: {:?}", output);
                str::from_utf8(&output.stdout).unwrap().trim().to_string()
            };
            println!("Coords: {}", output_str);

            println!("Test2");
            // If it could not detect any items in the inventory then go to stash and try again
            let output_str = if output_str == "Could not detect" {
                let output_stash = Command::new("python")
                    .arg("python_helpers/obj_detection.py")
                    .arg("images/stash.png")
                    .arg("F")
                    .arg("C")
                    .output()
                    .expect("Failed to execute command");

                println!("Test3");
                match enigo_functions::click_buton(&mut enigo, output_stash, true, 0, 0) {
                    Ok(_) => println!("Successfully clicked button!"),
                    Err(err) => println!("Got error while trying to click button: {:?}", err),
                }

                println!("Test4");
                let output_retry = Command::new("python")
                    .arg("python_helpers/multi_obj_detection_inv_stash.py")
                    .arg("temp_images/item/image.png")
                    .arg("SC")
                    .arg("F")
                    .arg("CR")
                    .output()
                    .expect("Failed to execute command");

                println!("Test5");
                // Convert the output bytes to a string
                str::from_utf8(&output_retry.stdout)
                    .unwrap()
                    .trim()
                    .to_string()
            } else {
                println!("Test6");
                output_str
            };

            println!("Test7");
            if output_str == "Could not detect" {
                println!("Test8");
                break 'add_items;
            }

            println!("Test9");
            // Split the string on newlines to get the list of coordinates
            let coords: Vec<&str> = output_str.split('\n').collect();

            println!("Test10");
            // Now, coords contains each of the coordinates
            for coord_str in coords.iter() {
                let mut rng = rand::thread_rng();

                if *coord_str == "Could not detect" || *coord_str == "" {
                    println!("Counld not find item");
                    // Moving away from items for obj detection purposes.
                    match enigo_functions::move_to_location_fast(
                        &mut enigo,
                        rng.gen_range(25..50),
                        rng.gen_range(200..300),
                        true,
                    ) {
                        Ok(_) => println!("Successfully moved to this location!"),
                        Err(err) => println!("Got error while trying to move cursor: {:?}", err),
                    }

                    // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
                    enigo.mouse_click(MouseButton::Left);
                    continue;
                }
                let coord: Vec<i32> = coord_str
                    .split_whitespace()
                    .map(|s| s.parse().expect("Failed to parse coordinate"))
                    .collect();

                println!("Test11");
                println!("Coords: {:?}", coords);
                println!("Coord: {:?}", coord);
                println!("Coord str: {}", coord_str);
                if coord.len() == 4 {
                    println!("Test12");
                    let (x1, y1, x2, y2) = (coord[0], coord[1], coord[2], coord[3]);

                    let mut rng = rand::thread_rng();

                    // Salt the pixels so that it does not click the same pixel every time.
                    let salt = rng.gen_range(-9..9);

                    // Gets the middle of the detected play button and clicks it
                    let middle_point_x = ((x2 - x1) / 2) + x1 + salt;
                    let middle_point_y = ((y2 - y1) / 2) + y1 + salt;

                    println!("Test13");
                    match enigo_functions::move_to_location_fast(
                        &mut enigo,
                        middle_point_x,
                        middle_point_y,
                        false,
                    ) {
                        Ok(_) => println!("Successfully moved to this location!"),
                        Err(err) => println!("Got error while trying to move cursor: {:?}", err),
                    }

                    println!("Test14");
                    // Tries to match every info image with the item and if there is a match then it will add it to the temporary vector variable.
                    for info_image in info_vec.iter() {
                        match download_image(info_image, "temp_images/info/image.png") {
                            Ok(_) => println!("Successfully downloaded info image"),
                            Err(err) => {
                                println!("Could not download image. Error \n{}", err);
                                return_to_lobby();
                                return Err(String::from("Player declined request"));
                            }
                        }

                        println!("Test15");
                        // SHOULD USE A VERSION OF OBJ DETECTION WITH A FASTER TIMEOUT. So that it wont wait for 4 minutes of there is no match
                        let output = Command::new("python")
                            .arg("python_helpers/obj_detection.py")
                            .arg("temp_images/info/image.png")
                            .arg("SF")
                            .arg("CR")
                            //.arg("C")
                            .output();

                        println!("Test16");
                        let output_unwrapped = output.unwrap(); // Bind `Output` to a variable to extend its lifetime
                        let output_str = str::from_utf8(&output_unwrapped.stdout).unwrap().trim();

                        println!("Test17");
                        if output_str != "Could not detect" {
                            println!("Found match!");
                            enigo.mouse_click(MouseButton::Left);
                            in_window_items.push((info_image, item));
                            item_limit += -1;
                            println!("Test18");
                        } else {
                            println!("Test19");
                            println!("No match. Checking next...");
                        }
                    }
                }
            }
        }

        println!("Test20");
        // Click checkbox to get into the confirmation trading window.
        let output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/trade_checkbox.png")
            .output()
            .expect("Failed to execute command");

        match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
            Ok(_) => println!("Successfully clicked button!"),
            Err(err) => println!("Got error while trying to click button: {:?}", err),
        }
        println!("Test21");

        // Check that the trader also has checked the checkbox and that we are now in the trading phase 2/2
        let output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/second_phase_check.png")
            .output();

        match output {
            Ok(_) => {
                println!("Trader accepted the trade!")
            }
            Err(_) => {
                println!("User did not accept trade.");
                // GO TO LOBBY
                return_to_lobby();
                return Err(String::from("User did not accept trade"));
            }
        }
        // Now the bot is in the double check trade window box.
        // Click the magnifying glasses on top of the items, incase the trader put anything in there
        let output = Command::new("python")
            .arg("python_helpers/inspect_items.py")
            .output()
            .expect("Failed to execute command");

        // Convert the output bytes to a string
        let output_str = str::from_utf8(&output.stdout).unwrap().trim();

        // Split the string on newlines to get the list of coordinates
        let coords: Vec<&str> = output_str.split('\n').collect();

        println!("Test22");
        // Now, coords contains each of the coordinates
        for coord_str in coords.iter() {
            let mut rng = rand::thread_rng();

            if *coord_str == "Could not detect" || *coord_str == "" {
                println!("Counld not find item");
                // Moving away from items for obj detection purposes.
                match enigo_functions::move_to_location_fast(
                    &mut enigo,
                    rng.gen_range(25..50),
                    rng.gen_range(200..300),
                    true,
                ) {
                    Ok(_) => println!("Successfully moved to this location!"),
                    Err(err) => println!("Got error while trying to move cursor: {:?}", err),
                }

                // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
                enigo.mouse_click(MouseButton::Left);
                continue;
            }
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

                println!("Test23");
                match enigo_functions::click_buton_direct(
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

        println!("Test24");
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
                    return_to_lobby();
                    return Err(String::from("Could not download image"));
                }
            }
            println!("Test25");

            // Using narrow version of multi obj detection.
            // Because the inventory/stash is still visable on this screen so the screenshot that the bot takes needs to be narrowed to only the trading window.

            // Handling output and avoiding temporary value drop issue
            let output_result = Command::new("python")
                .arg("python_helpers/multi_obj_detection_narrow.py")
                .arg("temp_images/item/image.png")
                .arg("SC")
                .arg("F")
                .arg("G")
                .arg("CR")
                .output();

            println!("Test26");
            match output_result {
                Ok(output) => {
                    println!("Test27");
                    let output_bytes = output.stdout;
                    let output_str = str::from_utf8(&output_bytes).unwrap().trim();
                    let coords: Vec<&str> = output_str.split('\n').collect();

                    // Now, coords contains each of the coordinates
                    for coord_str in coords.iter() {
                        let mut rng = rand::thread_rng();

                        if *coord_str == "Could not detect" || *coord_str == "" {
                            println!("Counld not find item");
                            // Moving away from items for obj detection purposes.
                            match enigo_functions::move_to_location_fast(
                                &mut enigo,
                                rng.gen_range(25..50),
                                rng.gen_range(200..300),
                                true,
                            ) {
                                Ok(_) => println!("Successfully moved to this location!"),
                                Err(err) => {
                                    println!("Got error while trying to move cursor: {:?}", err)
                                }
                            }

                            // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
                            enigo.mouse_click(MouseButton::Left);
                            continue;
                        }
                        let coord: Vec<i32> = coord_str
                            .split_whitespace()
                            .map(|s| s.parse().expect("Failed to parse coordinate"))
                            .collect();

                        println!("Test28");
                        if coord.len() == 4 {
                            println!("Test28");
                            let (x1, y1, x2, y2) = (coord[0], coord[1], coord[2], coord[3]);

                            let mut rng = rand::thread_rng();

                            // Salt the pixels so that it does not click the same pixel every time.
                            let salt = rng.gen_range(-9..9);

                            // Gets the middle of the detected play button and clicks it
                            let middle_point_x = ((x2 - x1) / 2) + x1 + salt;
                            let middle_point_y = ((y2 - y1) / 2) + y1 + salt;

                            println!("Test29");
                            match enigo_functions::move_to_location_fast(
                                &mut enigo,
                                middle_point_x,
                                middle_point_y,
                                true,
                            ) {
                                Ok(_) => println!("Successfully moved to this location!"),
                                Err(err) => {
                                    println!("Got error while trying to move cursor: {:?}", err)
                                }
                            }

                            println!("Test30");
                            // Tries to match every info image with the item and if there is a match then it will add it to the temporary vector variable.
                            for info_image in info_vec.iter() {
                                match download_image(info_image, "temp_images/info/image.png") {
                                    Ok(_) => println!("Successfully downloaded info image"),
                                    Err(err) => {
                                        println!("Could not download image. Error \n{}", err);
                                        return_to_lobby();
                                        return Err(String::from("Could not download image"));
                                    }
                                }

                                println!("Test31");
                                let output = Command::new("python")
                                    .arg("python_helpers/obj_detection.py")
                                    .arg("temp_images/info/image.png")
                                    .arg("SF")
                                    .arg("CR")
                                    .output();

                                println!("Test32");
                                match output {
                                    Ok(_) => {
                                        println!("Found match!");
                                        // Moving away from items for obj detection purposes.
                                        match enigo_functions::move_to_location_fast(
                                            &mut enigo,
                                            rng.gen_range(25..50),
                                            rng.gen_range(200..300),
                                            true,
                                        ) {
                                            Ok(_) => {
                                                println!("Successfully moved to this location!")
                                            }
                                            Err(err) => println!(
                                                "Got error while trying to move cursor: {:?}",
                                                err
                                            ),
                                        }

                                        // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
                                        enigo.mouse_click(MouseButton::Left);
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
                    println!("Test33");
                    println!("Could not find item. Cancelling trade and going to lobby..");
                    // GO TO LOBBY
                    return_to_lobby();
                    return Err(String::from("Could not find item"));
                }
            }
        }

        println!("Test34");
        // Check if trading_window_items is empty
        if in_window_items.is_empty() {
            println!("Test35");
            println!("No matches where found! Going back to lobby");
            return_to_lobby();
            return Err(String::from("No items found"));
        }
        // If the in_window_items is not emtpy then change the status of those images from "in escrow" to "traded"
        else {
            println!("Test36");
            // The bot ensures that the trade went through by making sure that it is the last link in the trade.
            // The bot waits for the trader to accept the trade by clicking the checkmark before the bot itself does.
            // Right as the trader clicks the button, the bot does as well, completing the trade for centain.
            // SHOULD USE A VERSION OF OBJ DETECTION WITH A FASTER TIMEOUT. So that it won't wait for 4 minutes if there is no match
            let output = Command::new("python")
                .arg("python_helpers/obj_detection.py")
                .arg("images/trader_ready.png")
                .output();

            println!("Test37");
            match output {
                Ok(_) => {
                    println!("User accepted trade!");
                    // Click the checkbox fast so that the other trader does not have time to decline in order to try to trick the bot.
                    let output = Command::new("python")
                        .arg("python_helpers/obj_detection.py")
                        .arg("images/trade_checkbox.png")
                        .output()
                        .expect("Failed to execute command");

                    println!("Test38");
                    // Convert the output into 4 coordinates and get the middle point of those.
                    // Then use the move_to_location_fast function to quickly move to the checkbox and click it
                    // Convert the output bytes to a string
                    let output_str = str::from_utf8(&output.stdout).unwrap().trim();

                    // Split the string on newlines to get the list of coordinates
                    let coords: Vec<&str> = output_str.split('\n').collect();

                    println!("Test39");
                    // Now, coords contains each of the coordinates
                    for coord_str in coords.iter() {
                        let mut rng = rand::thread_rng();

                        if *coord_str == "Could not detect" || *coord_str == "" {
                            println!("Counld not find item");
                            // Moving away from items for obj detection purposes.
                            match enigo_functions::move_to_location_fast(
                                &mut enigo,
                                rng.gen_range(25..50),
                                rng.gen_range(200..300),
                                true,
                            ) {
                                Ok(_) => println!("Successfully moved to this location!"),
                                Err(err) => {
                                    println!("Got error while trying to move cursor: {:?}", err)
                                }
                            }

                            // Click to avoid having selected an item. If not it might result in the item having the gold select border. This can cause the image detection to not detect the item.
                            enigo.mouse_click(MouseButton::Left);
                            continue;
                        }
                        let coord: Vec<i32> = coord_str
                            .split_whitespace()
                            .map(|s| s.parse().expect("Failed to parse coordinate"))
                            .collect();

                        println!("Test40");
                        if coord.len() == 4 {
                            let (x1, y1, x2, y2) = (coord[0], coord[1], coord[2], coord[3]);

                            let mut rng = rand::thread_rng();

                            // Salt the pixels so that it does not click the same pixel every time.
                            let salt = rng.gen_range(-9..9);

                            // Gets the middle of the detected play button and clicks it
                            let middle_point_x = ((x2 - x1) / 2) + x1 + salt;
                            let middle_point_y = ((y2 - y1) / 2) + y1 + salt;

                            println!("Test41");
                            // Now move to the middlepoint
                            match enigo_functions::move_to_location_fast(
                                &mut enigo,
                                middle_point_x,
                                middle_point_y,
                                true,
                            ) {
                                Ok(_) => println!("Successfully clicked button!"),
                                Err(err) => {
                                    println!("Got error while trying to click button: {:?}", err)
                                }
                            }

                            enigo.mouse_click(MouseButton::Left);
                        }
                    }
                }
                // Might not work...
                Err(_) => {
                    println!("Test42");
                    println!("User did not accept trade.");
                    // GO TO LOBBY
                    return_to_lobby();
                    return Err(String::from("User did not accept trade"));
                }
            }

            println!("Test43");
            println!("Changing the items statuses from 'in escrow' to 'traded'!");
            for (info_url, item_url) in in_window_items {
                match database_functions::set_item_status_by_urls(item_url, info_url, "returned") {
                    Ok(_) => println!("Changed the item status for 1 item!"),
                    Err(err) => println!("Got error while changing item status. Error: \n{}", err),
                }
            }
        }
        println!("Test44");

        return_to_lobby();
        Ok(String::from("Trade successful"))
    });
    match result_catch_panic {
        Ok(ok_result) => {
            match ok_result {
                Ok(message) => return Ok(message),
                Err(err) => {
                    return_to_lobby();
                    // Handle the string error here, maybe log it or convert it to your error type
                    return Err(err);
                }
            }
        }
        Err(panic_error) => {
            // This branch handles the case where panic::catch_unwind caught a panic
            // You can log the panic information, perform cleanup, or return an appropriate error
            // Convert panic_error (Box<dyn Any + Send>) to a suitable error type if needed
            return_to_lobby();
            // Return a generic error message
            return Err(format!(
                "Got error while trying to unwrap panic. Error: {:?}",
                panic_error
            ));
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
        .arg("F")
        .output()
        .expect("Failed to execute command");

    let output_str = str::from_utf8(&output.stdout).unwrap().trim();

    if output_str == "Could not detect" {
        return_to_lobby();
        return Err("Trader was not present in trading channel");
    }

    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Now enter bards trading post
    // Run the trade channel button detector
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/utility_trade.png")
        .arg("F")
        .output()
        .expect("Failed to execute command");

    let output_str = str::from_utf8(&output.stdout).unwrap().trim();

    if output_str == "Could not detect" {
        return_to_lobby();
        return Err("Trader was not present in trading channel");
    }

    match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Check if the player is in the bard trading channel
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/present_trader.png")
        .arg("F")
        .output()
        .expect("Failed to execute command");

    let output_str = str::from_utf8(&output.stdout).unwrap().trim();

    if output_str == "Could not detect" {
        return_to_lobby();
        return Err("Trader was not present in trading channel");
    }

    //It now sends a trade to the player
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/find_id.png")
        .arg("F")
        .output()
        .expect("Failed to execute command");

    let output_str = str::from_utf8(&output.stdout).unwrap().trim();

    if output_str == "Could not detect" {
        return_to_lobby();
        return Err("Trader was not present in trading channel");
    }

    // Search after the trader in the trade tab
    match enigo_functions::click_buton(&mut enigo, output, true, 0, -33) {
        Ok(_) => println!("Successfully clicked button!"),
        Err(err) => println!("Got error while trying to click button: {:?}", err),
    }

    // Clear the search bar so that it is ready for a new id.
    enigo.key_sequence_parse("{+CONTROL}a{-CONTROL}{+DELETE}{-DELETE}");

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
        .arg("F")
        .output();

    let user_is_in_trade = match &output {
        Ok(output) => {
            let output_str = str::from_utf8(&output.stdout).unwrap().trim();

            if output_str == "Could not detect" {
                false
            } else {
                true
            }
        }
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

    // Check if we are in the trading window.
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/trade_screen_identifier.png")
        .arg("C")
        .arg("F")
        .output()
        .expect("Failed to execute command");

    let output_str = str::from_utf8(&output.stdout).unwrap().trim();

    if output_str != "Could not detect" {
        println!("Successfully clicked button!");
        return Ok("User accepted trade");
    } else {
        println!("Could not detect trading window");
        return_to_lobby();
        return Err("Could not detect trading window");
    }
}
pub fn return_to_lobby() {
    let mut enigo = Enigo::new();

    // Check if in play tab
    // If yes then return
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/in_play_tab.png")
        .arg("SF")
        .arg("C")
        .output()
        .expect("Failed to execute command");

    let output_str = str::from_utf8(&output.stdout).unwrap().trim();

    if output_str != "Could not detect" {
        println!("Already in play tab");
        return;
    }

    // Check if can go to play tab
    // If yes then return
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/play_tab.png")
        .arg("SF")
        .output()
        .expect("Failed to execute command");

    let output_str = str::from_utf8(&output.stdout).unwrap().trim();

    if output_str != "Could not detect" {
        match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
            Ok(_) => {
                println!("Successfully clicked button!");
                return;
            }
            Err(err) => println!("Got error while trying to click button: {:?}", err),
        }
    }

    // Run this code twice.
    // It presses escape and clicks yes.
    // If we are in an active trade then it will only go out of the active trade and into the trading channel.
    // If we are in the trading channel then it will go out and into the "trade" tab
    // This loop will run twice if we are in an active trade and only run once if the bot is in a trading channel
    // When the loop is done the bot should have returned to the lobby and be ready again.
    // If not then it will continue and restart the game.
    for x in 0..2 {
        // Press escape and press button "yes"
        enigo.key_click(Key::Escape);

        let output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/leave_post.png")
            .arg("F")
            .arg("C")
            .output()
            .expect("Failed to execute command");

        let output_str = str::from_utf8(&output.stdout).unwrap().trim();

        if output_str != "Could not detect" {
            match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
                Ok(_) => {
                    println!("Successfully clicked button!");
                }
                Err(err) => println!("Got error while trying to click button: {:?}", err),
            }
        }
        // In the first itteration check very fast as it is unlikely to be out yet
        // Then at the second itteration check it slower to account for the popup delay
        let super_fast: &str;
        if x > 0 {
            super_fast = "SF";
        } else {
            super_fast = "F";
        }
        // Check if can go to play tab
        // If yes then go and return
        let output = Command::new("python")
            .arg("python_helpers/obj_detection.py")
            .arg("images/play_tab.png")
            .arg(super_fast)
            .output()
            .expect("Failed to execute command");

        let output_str = str::from_utf8(&output.stdout).unwrap().trim();

        if output_str != "Could not detect" {
            match enigo_functions::click_buton(&mut enigo, output, true, 0, 0) {
                Ok(_) => {
                    println!("Successfully clicked button!");
                    return;
                }
                Err(err) => println!("Got error while trying to click button: {:?}", err),
            }
        }
    }
    // Press windows key and go to blacksmith launcher
    // Minize game
    enigo.key_sequence_parse("{+META}m{-META}");

    // Press "Stop" button
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/stop_game.png")
        .arg("F")
        .output()
        .expect("Failed to execute command");

    let output_str = str::from_utf8(&output.stdout).unwrap().trim();

    if output_str != "Could not detect" {
        match enigo_functions::click_buton(&mut enigo, output, false, 0, 0) {
            Ok(_) => {
                println!("Successfully clicked button!");
                // Press "Ok" button to confirm closing game
                let output = Command::new("python")
                    .arg("python_helpers/obj_detection.py")
                    .arg("images/close_game_ok.png")
                    .arg("F")
                    .output()
                    .expect("Failed to execute command");

                let output_str = str::from_utf8(&output.stdout).unwrap().trim();

                if output_str != "Could not detect" {
                    match enigo_functions::click_buton(&mut enigo, output, false, 0, 0) {
                        Ok(_) => {
                            println!("Successfully clicked button!");
                        }
                        Err(err) => println!("Got error while trying to click button: {:?}", err),
                    }
                }
            }
            Err(err) => println!("Got error while trying to click button: {:?}", err),
        }
    }
    // Run open_game_go_to_lobby
    tokio::spawn(async move {
        open_game_go_to_lobby_no_state_change(false).await;
    });
    // Return
    return;
}

fn download_image(url: &str, save_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure the 'temp_images' directory exists
    if !Path::new("temp_images").exists() {
        fs::create_dir("temp_images")?;
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

    // Adaptive wait
    let max_attempts = 20;
    for _ in 0..max_attempts {
        let metadata = fs::metadata(save_path);
        if let Ok(meta) = metadata {
            if meta.len() > 0 && meta.is_file() {
                // File exists and has content, break out of the loop
                break;
            }
        }
        // Sleep for a short interval before checking again
        sleep(Duration::from_millis(100));
    }

    // Optionally, after the loop, you can check one final time and
    // return an error if the file is still not ready.
    let metadata = fs::metadata(save_path);
    if metadata.is_err() || metadata.unwrap().len() == 0 {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "File is not ready after adaptive wait",
        )));
    }

    Ok(())
}

fn calculate_pouches(
    owe: i32,
    inventory_50: i32,
    stash_50: i32,
    inventory_35: i32,
    stash_35: i32,
) -> (i32, i32, i32, i32) {
    let mut used_inventory_50 = 0;
    let mut used_stash_50 = 0;
    let mut used_inventory_35 = 0;
    let mut used_stash_35 = 0;
    let mut owe = owe;

    // Use 50-coin packs from the inventory first
    while owe >= 50 && used_inventory_50 < inventory_50 {
        owe -= 50;
        used_inventory_50 += 1;
    }

    // Use 50-coin packs from the stash next
    while owe >= 50 && used_stash_50 < stash_50 {
        owe -= 50;
        used_stash_50 += 1;
    }

    // Adjust for potential better configurations with extra 50-coin packs
    let mut best_owe = owe;
    let mut best_inventory_50 = used_inventory_50;
    let mut best_stash_50 = used_stash_50;
    while used_inventory_50 + used_stash_50 < inventory_50 + stash_50 {
        owe -= 50;
        if used_inventory_50 < inventory_50 {
            used_inventory_50 += 1;
        } else {
            used_stash_50 += 1;
        }
        let current_difference = owe.abs();
        if current_difference <= 20 {
            best_owe = owe;
            best_inventory_50 = used_inventory_50;
            best_stash_50 = used_stash_50;
            break;
        }
    }

    owe = best_owe;
    used_inventory_50 = best_inventory_50;
    used_stash_50 = best_stash_50;

    // Use 35-coin packs from the inventory
    while (owe > 20 || owe < -20) && used_inventory_35 < inventory_35 {
        owe -= 35;
        used_inventory_35 += 1;
    }

    // Use 35-coin packs from the stash
    while (owe > 20 || owe < -20) && used_stash_35 < stash_35 {
        owe -= 35;
        used_stash_35 += 1;
    }

    let total_paid =
        (used_inventory_50 + used_stash_50) * 50 + (used_inventory_35 + used_stash_35) * 35;
    println!("Total amount paid back: {} coins", total_paid);
    println!(
        "Used {} packs of 50 from inventory and {} from stash",
        used_inventory_50, used_stash_50
    );
    println!(
        "Used {} packs of 35 from inventory and {} from stash",
        used_inventory_35, used_stash_35
    );

    (
        used_inventory_50,
        used_stash_50,
        used_inventory_35,
        used_stash_35,
    )
}

// Clicks on pouches
fn click_pouches(
    coords_pouches: &str,
    pouch_count: i32,
    mut clicked_pouches: i32,
    max_clicked: i32,
) -> i32 {
    if coords_pouches == "Could not detect" {
        return clicked_pouches;
    }

    let mut enigo = Enigo::new();

    let coords: Vec<&str> = coords_pouches.split('\n').collect();

    println!("coords: {:?}", coords);
    // Now, coords contains each of the coordinates
    for (i, coord_str) in coords.iter().enumerate() {
        // Check if no pouches should be used before clicking on them
        if pouch_count < 1 {
            break;
        }
        // Check if the for loop has clicked on the amount of bags that it was supposed too
        if i as i32 >= pouch_count {
            break;
        }
        if clicked_pouches >= max_clicked {
            break;
        }

        let coord: Vec<i32> = coord_str
            .split_whitespace()
            .map(|s| s.parse().expect("Failed to parse coordinate"))
            .collect();

        if coord.len() == 4 {
            let (x1, y1, x2, y2) = (coord[0], coord[1], coord[2], coord[3]);

            let mut rng = rand::thread_rng();

            // Salt the pixels so that it does not click the same pixel every time.
            let salt = rng.gen_range(-9..9);

            // Gets the middle of the detected gold pouch and salt it to not click the same pixel every time
            let middle_point_x = ((x2 - x1) / 2) + x1 + salt;
            let middle_point_y = ((y2 - y1) / 2) + y1 + salt;

            match enigo_functions::move_to_location_fast(
                &mut enigo,
                middle_point_x,
                middle_point_y,
                false,
            ) {
                Ok(_) => println!("Successfully moved to this location!"),
                Err(err) => println!("Got error while trying to move cursor: {:?}", err),
            }
            // Now we are hovering over a money pouch, click on it to add it to the trade
            // Adding delay here else game might not be able to keep up with clicking so fast
            sleep(Duration::from_millis(300));
            enigo.mouse_click(MouseButton::Left);
            clicked_pouches += 1;
        }
    }
    clicked_pouches
}

// Needed for return to lobby as return to lobby cannot have the bot_info param as it interfers too much with other functions
pub async fn open_game_go_to_lobby_no_state_change(start_launcher: bool) {
    let enigo = Arc::new(Mutex::new(Enigo::new()));

    println!("Opening game!");
    //tokio::time::sleep(tokio::time::Duration::from_secs(10000)).await;

    let mut enigo = enigo.lock().unwrap();

    // If the launcher is already open, for example if we are restarting the game, then we do not need to open the launcher again.
    if start_launcher {
        // Minimizes all tabs so that only the game is opened. To avoid clicking on other tabs
        enigo.key_sequence_parse("{+META}m{-META}");

        // Start the launcher
        start_game(&mut enigo, "blacksmith");
    }

    // Quickly check if the game needs to update
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/update.png")
        .arg("SF")
        .output()
        .expect("Failed to execute command");

    // Convert the output bytes to a string
    let output_str = str::from_utf8(&output.stdout).unwrap().trim();

    // If the update menu was found then click the update button
    if output_str != "Could not find" {
        match enigo_functions::click_buton(&mut enigo, output, false, 0, 0) {
            Ok(_) => println!("Successfully clicked button!"),
            Err(err) => println!("Got error while trying to click button: {:?}", err),
        }
    }

    // Run the launcher play button detector
    let output = Command::new("python")
        .arg("python_helpers/obj_detection.py")
        .arg("images/play.png")
        .arg("F")
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
}
