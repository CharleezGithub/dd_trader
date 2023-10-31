use rusqlite::{params, Connection, Result};
use std::sync::{Arc, Mutex};

use crate::{Trader, TradersContainer};

pub fn get_links_for_user(channel_id: &str, user_id: &str) -> Result<(Vec<String>, Vec<String>)> {
    let conn = Connection::open("C:/Users/Alex/Desktop/VSCode/dd_trader/trading_bot.db")?;

    // Retrieve trader_id using discord_id (user_id)
    let mut stmt = conn.prepare(
        "
        SELECT id 
        FROM traders 
        WHERE discord_id = ?1
        ",
    )?;
    let mut rows = stmt.query(params![user_id])?;

    let trader_id: i64 = if let Some(row) = rows.next()? {
        row.get(0)?
    } else {
        return Err(rusqlite::Error::QueryReturnedNoRows); // Or consider providing a custom error message
    };

    // Fetch item and info links using trader_id
    let mut stmt = conn.prepare(
        "
        SELECT items.item_image_url, items.info_image_url
        FROM items
        JOIN trades ON items.trade_id = trades.id
        WHERE trades.channel_id = ?1
        AND items.trader_id = ?2
    ",
    )?;

    let rows = stmt.query_map(params![channel_id, trader_id], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?;

    let mut item_links = Vec::new();
    let mut info_links = Vec::new();

    for link_result in rows {
        let (item_link, info_link) = link_result?;
        item_links.push(item_link);
        info_links.push(info_link);
    }

    Ok((item_links, info_links))
}

pub fn has_paid_fee(channel_id: &str, user_id: &str) -> Result<bool> {
    let conn = Connection::open("C:/Users/Alex/Desktop/VSCode/dd_trader/trading_bot.db")?;

    // Retrieve trader_id using discord_id (user_id)
    let mut stmt = conn.prepare(
        "
        SELECT id 
        FROM traders 
        WHERE discord_id = ?1
        ",
    )?;
    let mut rows = stmt.query(params![user_id])?;

    let trader_id: i64 = if let Some(row) = rows.next()? {
        row.get(0)?
    } else {
        return Ok(false); // Or consider Err to indicate that user is not found
    };

    // Check the paid status in trades using trader_id
    let mut stmt = conn.prepare(
        "
        SELECT trader1_paid, trader2_paid
        FROM trades
        WHERE channel_id = ?1 AND (trader1_id = ?2 OR trader2_id = ?2)
        ",
    )?;

    let mut rows = stmt.query(params![channel_id, trader_id])?;

    if let Some(row) = rows.next()? {
        // Check which trader the user is (trader1 or trader2) and retrieve the corresponding paid status
        let (trader1_paid, trader2_paid): (bool, bool) = (row.get(0)?, row.get(1)?);

        // If user is trader1, return trader1_paid, else return trader2_paid
        return Ok(trader1_paid || trader2_paid);
    }

    Ok(false)
}

pub fn cancel_trade_check(discord_id: &str, channel_id: &str) -> Result<bool> {
    let conn = Connection::open("C:/Users/Alex/Desktop/VSCode/dd_trader/trading_bot.db")?;

    // True if trader1, false if trader2
    let mut trader_1_or_2 = true;

    let mut gold_in_escrow = false;

    let mut items_in_escrow = false;

    // Find out if trader_id is trader1 or trader2
    // Get trader's internal ID using discord_id (user_id)
    let trader_id: i32 = conn.query_row(
        "
        SELECT id
        FROM traders
        WHERE discord_id = ?1
        ",
        params![discord_id],
        |row| row.get(0),
    )?;
    let mut stmt = conn.prepare(
        "
        SELECT trader1_id, trader2_id
        FROM trades
        WHERE channel_id = ?1
        ",
    )?;

    let mut rows = stmt.query(params![channel_id])?;
    
    if let Some(row) = rows.next()? {
        let (trader1_id, trader2_id): (i32, i32) = (row.get(0)?, row.get(1)?);

        if trader_id == trader1_id {
            trader_1_or_2 = true;
        }
        else if trader_id == trader2_id {
            trader_1_or_2 = false;
            
        }
    }

    
    // Check if any gold has been claimed and
    // Check if there is any gold left to 
    let mut stmt = conn.prepare(
        "
        SELECT trader1_gold_received, trader2_gold_received, trader1_gold_traded, trader2_gold_traded
        FROM trades
        WHERE channel_id = ?1
        ",
    )?;

    let mut rows = stmt.query(params![channel_id])?;

    if let Some(row) = rows.next()? {
        let (trader1_gold_claimed, trader2_gold_claimed, trader1_gold_traded, trader2_gold_traded): (i32, i32, i32, i32) = (row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?);

        if trader_1_or_2 || trader1_gold_traded > 30 {
            gold_in_escrow = true;
        }
        else if !trader_1_or_2 || trader2_gold_traded > 30 {
            gold_in_escrow = true;
        }

        if trader1_gold_claimed > 30 || trader2_gold_claimed > 30 {
            return Ok(false);
        }
    }

    // Check if any items have been claimed
    // Get trader1_id and trader2_id
    let mut stmt = conn.prepare(
        "
        SELECT trader1_id, trader2_id
        FROM trades
        WHERE channel_id = ?1
        ",
    )?;

    let mut rows = stmt.query(params![channel_id])?;

    if let Some(row) = rows.next()? {
        let (trader1_id, trader2_id): (i32, i32) = (row.get(0)?, row.get(1)?);

        // Check if at least one item or over 30 gold have been traded to the bot
        // Else there is no reason to send anything back because there is nothing to send
        let trader1_item_escorw_count: i32 = conn.query_row(
            "SELECT COUNT(*)
        FROM items
        JOIN trades ON items.trade_id = trades.id
        JOIN traders ON items.trader_id = traders.id
        WHERE items.status = 'in_escrow'
        AND trades.channel_id = ?1
        AND traders.discord_id = ?2",
            params![channel_id, trader1_id],
            |row| row.get(0),
        )?;
        let trader2_item_escorw_count: i32 = conn.query_row(
            "SELECT COUNT(*)
        FROM items
        JOIN trades ON items.trade_id = trades.id
        JOIN traders ON items.trader_id = traders.id
        WHERE items.status = 'in_escrow'
        AND trades.channel_id = ?1
        AND traders.discord_id = ?2",
            params![channel_id, trader2_id],
            |row| row.get(0),
        )?;

        if trader_1_or_2 || trader1_item_escorw_count > 0 {
            items_in_escrow = true;
        }
        else if !trader_1_or_2 || trader2_item_escorw_count > 0 {
            items_in_escrow = true;
        }


        let trader1_count: i32 = conn.query_row(
            "SELECT COUNT(*)
            FROM items
            JOIN trades ON items.trade_id = trades.id
            JOIN traders ON items.trader_id = traders.id
            WHERE items.status = 'traded'
            AND trades.channel_id = ?1
            AND traders.discord_id = ?2",
            params![channel_id, trader1_id],
            |row| row.get(0),
        )?;
        let trader2_count: i32 = conn.query_row(
            "SELECT COUNT(*)
            FROM items
            JOIN trades ON items.trade_id = trades.id
            JOIN traders ON items.trader_id = traders.id
            WHERE items.status = 'traded'
            AND trades.channel_id = ?1
            AND traders.discord_id = ?2",
            params![channel_id, trader2_id],
            |row| row.get(0),
        )?;
        if trader1_count > 0 || trader2_count > 0 {
            return Ok(false);
        }
    }
    if !gold_in_escrow || !items_in_escrow{
        return Ok(false);
    }
    Ok(true)
}

pub fn get_gold_for_user(channel_id: &str, user_id: &str) -> Result<i32> {
    let conn = Connection::open("C:/Users/Alex/Desktop/VSCode/dd_trader/trading_bot.db")?;

    // Get trader's internal ID using discord_id (user_id)
    let trader_id: i32 = conn.query_row(
        "
        SELECT id
        FROM traders
        WHERE discord_id = ?1
        ",
        params![user_id],
        |row| row.get(0),
    )?;

    // Get gold values and trader IDs from the trades table
    let mut stmt = conn.prepare(
        "
        SELECT trader1_gold, trader2_gold, trader1_id, trader2_id 
        FROM trades 
        WHERE channel_id = ?1
        ",
    )?;

    let mut rows = stmt.query(params![channel_id])?;

    if let Some(row) = rows.next()? {
        let (trader1_gold, trader2_gold, trader1_id, trader2_id): (i32, i32, i32, i32) =
            (row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?);

        // Determine whether the user is trader1 or trader2, and return the corresponding gold amount
        return Ok(if trader_id == trader1_id {
            trader1_gold
        } else if trader_id == trader2_id {
            trader2_gold
        } else {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        });
    }

    Err(rusqlite::Error::QueryReturnedNoRows)
}

pub fn set_gold_fee_status(
    channel_id: &str,
    user_id: &str,
    has_paid: bool,
) -> Result<(), rusqlite::Error> {
    let conn = Connection::open("C:/Users/Alex/Desktop/VSCode/dd_trader/trading_bot.db")?;

    // Identify the trader id for the user
    let mut stmt = conn.prepare(
        "
        SELECT id 
        FROM traders 
        WHERE discord_id = ?1
        ",
    )?;
    let mut rows = stmt.query(params![user_id])?;

    let trader_id: i64 = if let Some(row) = rows.next()? {
        row.get(0)?
    } else {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    };

    // Identify whether the user is trader1 or trader2 in the trade
    let mut stmt = conn.prepare(
        "
        SELECT trader1_id, trader2_id 
        FROM trades 
        WHERE channel_id = ?1
        ",
    )?;
    let mut rows = stmt.query(params![channel_id])?;

    if let Some(row) = rows.next()? {
        let (trader1_id, trader2_id): (i64, i64) = (row.get(0)?, row.get(1)?);

        // Determine which trader the user is and update the corresponding paid status
        if trader_id == trader1_id {
            let mut stmt = conn.prepare(
                "
                UPDATE trades 
                SET trader1_paid = ?2 
                WHERE channel_id = ?1
                ",
            )?;
            stmt.execute(params![channel_id, has_paid])?;
        } else if trader_id == trader2_id {
            let mut stmt = conn.prepare(
                "
                UPDATE trades 
                SET trader2_paid = ?2 
                WHERE channel_id = ?1
                ",
            )?;
            stmt.execute(params![channel_id, has_paid])?;
        } else {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
    } else {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    Ok(())
}

pub fn set_item_status_by_urls(
    item_image_url: &str,
    info_image_url: &str,
    new_status: &str,
) -> Result<()> {
    let conn = Connection::open("C:/Users/Alex/Desktop/VSCode/dd_trader/trading_bot.db")?;

    let mut stmt = conn.prepare(
        "
        UPDATE items 
        SET status = ?3 
        WHERE item_image_url = ?1 AND info_image_url = ?2
    ",
    )?;

    let rows_affected = stmt.execute(params![item_image_url, info_image_url, new_status])?;

    if rows_affected == 0 {
        Err(rusqlite::Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}

pub fn populate_traders_from_db(traders_container: &Arc<Mutex<TradersContainer>>) -> Result<()> {
    let mut traders = traders_container.lock().unwrap();

    let conn = Connection::open("C:/Users/Alex/Desktop/VSCode/dd_trader/trading_bot.db")?;

    let mut stmt = conn.prepare(
        "
        SELECT 
            t.id, t.discord_id, tr.channel_id, 
            CASE WHEN t.id = tr.trader1_id THEN tr.trader1_gold ELSE tr.trader2_gold END,
            CASE WHEN t.id = tr.trader1_id THEN tr.trader1_paid ELSE tr.trader2_paid END,
            i.item_image_url, i.info_image_url
        FROM traders t
        JOIN trades tr ON t.id = tr.trader1_id OR t.id = tr.trader2_id
        LEFT JOIN items i ON tr.id = i.trade_id AND t.id = i.trader_id
        WHERE tr.status = 'ongoing'
    ",
    )?;

    let rows = stmt.query_map(params![], |row| {
        Ok((
            row.get::<_, String>(1)?,         // discord_id
            row.get::<_, String>(2)?,         // channel_id
            row.get::<_, i32>(3)?,            // gold
            row.get::<_, bool>(4)?,           // has_paid_gold_fee
            row.get::<_, Option<String>>(5)?, // item_image_url
            row.get::<_, Option<String>>(6)?, // info_image_url
        ))
    })?;

    let mut traders_map: std::collections::HashMap<
        (String, String),
        (Vec<String>, Vec<String>, i32, bool),
    > = std::collections::HashMap::new();
    for row in rows {
        if let Ok((
            discord_id,
            channel_id,
            gold,
            has_paid_gold_fee,
            item_image_url,
            info_image_url,
        )) = row
        {
            let entry = traders_map
                .entry((discord_id.clone(), channel_id.clone()))
                .or_insert((Vec::new(), Vec::new(), gold, has_paid_gold_fee));
            if let (Some(item_image_url), Some(info_image_url)) = (item_image_url, info_image_url) {
                entry.0.push(item_image_url);
                entry.1.push(info_image_url);
            }
        }
    }

    for ((discord_id, channel_id), (item_images, info_images, gold, has_paid_gold_fee)) in
        traders_map
    {
        traders.append(Trader {
            in_game_id: "".to_string(), // Empty, as this will be assigned later
            discord_channel_id: channel_id,
            discord_id,
            item_images,
            info_images,
            gold,
            has_paid_gold_fee,
        });
    }

    Ok(())
}

pub fn items_in_escrow_count(trader: &Trader) -> Result<i32> {
    let channel_id = &trader.discord_channel_id;
    let discord_id = &trader.discord_id;

    let conn = Connection::open("C:/Users/Alex/Desktop/VSCode/dd_trader/trading_bot.db")?;

    let count: i32 = conn.query_row(
        "SELECT COUNT(*)
        FROM items
        JOIN trades ON items.trade_id = trades.id
        JOIN traders ON items.trader_id = traders.id
        WHERE items.status = 'in escrow'
        AND trades.channel_id = ?1
        AND traders.discord_id = ?2",
        params![channel_id, discord_id],
        |row| row.get(0),
    )?;

    Ok(count)
}

pub fn add_gold_to_trader(
    channel_id: &String,
    discord_id: &String,
    gold_to_add: i32,
) -> Result<()> {
    let conn = Connection::open("C:/Users/Alex/Desktop/VSCode/dd_trader/trading_bot.db")?;

    // Determine whether the user is trader1 or trader2 in the channel
    let trader_role_result: Result<String> = conn.query_row(
        "SELECT 
            CASE 
                WHEN trader1_id = (SELECT id FROM traders WHERE discord_id = ?1) THEN 'trader1'
                WHEN trader2_id = (SELECT id FROM traders WHERE discord_id = ?1) THEN 'trader2'
                ELSE NULL
            END AS trader_role
        FROM trades 
        WHERE channel_id = ?2",
        params![discord_id, channel_id],
        |row| row.get(0),
    );

    match trader_role_result {
        Ok(role) => {
            // Determine which trader's gold needs to be updated (trader1_gold_traded or trader2_gold_traded)
            let gold_column = if &role == "trader1" {
                "trader1_gold_traded"
            } else {
                "trader2_gold_traded"
            };

            conn.execute(
                &format!(
                    "UPDATE trades 
                    SET {} = {} + ?1 
                    WHERE channel_id = ?2",
                    gold_column, gold_column
                ),
                params![gold_to_add, channel_id],
            )?;

            Ok(())
        }
        Err(_) => Err(rusqlite::Error::QueryReturnedNoRows),
    }
}

pub fn add_gold_received_to_trader(
    channel_id: &String,
    discord_id: &String,
    gold_to_add: i32,
) -> Result<()> {
    let conn = Connection::open("C:/Users/Alex/Desktop/VSCode/dd_trader/trading_bot.db")?;

    // Determine whether the user is trader1 or trader2 in the channel
    let trader_role_result: Result<String> = conn.query_row(
        "SELECT 
            CASE 
                WHEN trader1_id = (SELECT id FROM traders WHERE discord_id = ?1) THEN 'trader1'
                WHEN trader2_id = (SELECT id FROM traders WHERE discord_id = ?1) THEN 'trader2'
                ELSE NULL
            END AS trader_role
        FROM trades 
        WHERE channel_id = ?2",
        params![discord_id, channel_id],
        |row| row.get(0),
    );

    match trader_role_result {
        Ok(role) => {
            // Determine which trader's gold needs to be updated (trader1_gold_traded or trader2_gold_traded)
            let gold_column = if &role == "trader1" {
                "trader1_gold_received"
            } else {
                "trader2_gold_received"
            };

            conn.execute(
                &format!(
                    "UPDATE trades 
                    SET {} = {} + ?1 
                    WHERE channel_id = ?2",
                    gold_column, gold_column
                ),
                params![gold_to_add, channel_id],
            )?;

            Ok(())
        }
        Err(_) => Err(rusqlite::Error::QueryReturnedNoRows),
    }
}
