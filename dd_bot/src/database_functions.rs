#![allow(dead_code)]

use rusqlite::{params, Connection, Result};
use std::sync::{Arc, Mutex};

use crate::{Trader, TradersContainer};

// Not used as of right now, due to the "populate traders from db" function already retrieving this information and storing it in the traders container.
pub fn get_links_for_user(channel_id: &str, user_id: &str) -> Result<(Vec<String>, Vec<String>)> {
    let conn = Connection::open("../trading_bot.db")?;

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
    let conn = Connection::open("../trading_bot.db")?;

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

// Not used as of right now, due to the "populate traders from db" function already retrieving this information and storing it in the traders container.
pub fn get_gold_for_user(channel_id: &str, user_id: &str) -> Result<i32> {
    let conn = Connection::open("../trading_bot.db")?;

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
    let conn = Connection::open("../trading_bot.db")?;

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
    let conn = Connection::open("../trading_bot.db")?;

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

    let conn = Connection::open("../trading_bot.db")?;

    let mut stmt = conn.prepare(
        "
        SELECT 
            t.id, t.discord_id, tr.channel_id, 
            CASE WHEN t.id = tr.trader1_id THEN tr.trader1_gold ELSE tr.trader2_gold END,
            CASE WHEN t.id = tr.trader1_id THEN tr.trader1_paid ELSE tr.trader2_paid END,
            not_traded.item_image_url, not_traded.info_image_url,
            escrow.item_image_url, escrow.info_image_url,
            traded.item_image_url, traded.info_image_url
        FROM traders t
        JOIN trades tr ON t.id = tr.trader1_id OR t.id = tr.trader2_id
        LEFT JOIN items not_traded ON tr.id = not_traded.trade_id AND t.id = not_traded.trader_id AND not_traded.status = 'not traded'
        LEFT JOIN items escrow ON tr.id = escrow.trade_id AND t.id = escrow.trader_id AND escrow.status = 'in escrow'
        LEFT JOIN items traded ON tr.id = traded.trade_id AND t.id = traded.trader_id AND traded.status = 'traded'
        WHERE tr.status = 'ongoing' OR tr.status = 'canceled'
    ",
    )?;

    let rows = stmt.query_map(params![], |row| {
        Ok((
            row.get::<_, String>(1)?,          // discord_id
            row.get::<_, String>(2)?,          // channel_id
            row.get::<_, i32>(3)?,             // gold
            row.get::<_, bool>(4)?,            // has_paid_gold_fee
            row.get::<_, Option<String>>(5)?,  // item_image_url, "not traded" status
            row.get::<_, Option<String>>(6)?,  // info_image_url, "not trade" status
            row.get::<_, Option<String>>(7)?,  // item_image_url, "in escrow" status
            row.get::<_, Option<String>>(8)?,  // info_image_url, "in escrow" status
            row.get::<_, Option<String>>(9)?,  // item_image_url, "traded" status
            row.get::<_, Option<String>>(10)?, // info_image_url, "traded" status
        ))
    })?;

    let mut traders_map: std::collections::HashMap<
        (String, String),
        (
            Vec<String>,
            Vec<String>,
            Vec<String>,
            Vec<String>,
            Vec<String>,
            Vec<String>,
            i32,
            bool,
        ),
    > = std::collections::HashMap::new();

    for row in rows {
        if let Ok((
            discord_id,
            channel_id,
            gold,
            has_paid_gold_fee,
            item_image_url_not_traded,
            info_image_url_not_traded,
            item_image_url_escrow,
            info_image_url_escrow,
            item_image_url_traded,
            info_image_url_traded,
        )) = row
        {
            let entry = traders_map
                .entry((discord_id.clone(), channel_id.clone()))
                .or_insert((
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    gold,
                    has_paid_gold_fee,
                ));

            if let Some(url) = item_image_url_not_traded {
                entry.0.push(url);
            }
            if let Some(url) = info_image_url_not_traded {
                entry.1.push(url);
            }
            if let Some(url) = item_image_url_escrow {
                entry.2.push(url);
            }
            if let Some(url) = info_image_url_escrow {
                entry.3.push(url);
            }
            if let Some(url) = item_image_url_traded {
                entry.4.push(url);
            }
            if let Some(url) = info_image_url_traded {
                entry.5.push(url);
            }
        }
    }

    for (
        (discord_id, channel_id),
        (
            item_images_not_traded,
            info_images_not_traded,
            item_images_escrow,
            info_images_escrow,
            item_images_traded,
            info_images_traded,
            gold,
            has_paid_gold_fee,
        ),
    ) in traders_map
    {
        traders.append(Trader {
            in_game_id: "".to_string(), // Empty, as this will be assigned later
            discord_channel_id: channel_id,
            discord_id,
            item_images_not_traded,
            item_images_escrow,
            item_images_traded,
            info_images_not_traded,
            info_images_escrow,
            info_images_traded,
            gold,
            has_paid_gold_fee,
        });
    }

    Ok(())
}

pub fn items_in_escrow_count(trader: &Trader) -> Result<i32> {
    let channel_id = &trader.discord_channel_id;
    let discord_id = &trader.discord_id;

    let conn = Connection::open("../trading_bot.db")?;

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

// This function does 2 things.
// It both adds the specified gold amount to the "trader_gold_traded" and it also subtracts that same amount from the "trader_gold" value for that same trader.
pub fn add_gold_to_trader(
    channel_id: &String,
    discord_id: &String,
    gold_to_add: i32,
) -> Result<()> {
    let conn = Connection::open("../trading_bot.db")?;

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
            let gold_traded_column = if &role == "trader1" {
                "trader1_gold_traded"
            } else {
                "trader2_gold_traded"
            };
            let gold_column = if &role == "trader1_gold_traded" {
                "trader1_gold"
            } else {
                "trader2_gold"
            };

            conn.execute(
                &format!(
                    "UPDATE trades 
                    SET {} = {} - ?1 
                    SET {} = {} + ?1 
                    WHERE channel_id = ?2",
                    gold_column, gold_column, gold_traded_column, gold_traded_column
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
    let conn = Connection::open("../trading_bot.db")?;

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
