use rusqlite::{params, Connection, Result};

pub fn get_links_for_user(channel_id: &str, user_id: &str) -> Result<(Vec<String>, Vec<String>)> {
    let conn = Connection::open("C:/Users/Alex/Desktop/VSCode/dd_trader/trading_bot.db")?;

    let mut stmt = conn.prepare(
        "
        SELECT items.item_image_url, items.info_image_url
        FROM items
        JOIN trades ON items.trade_id = trades.id
        WHERE trades.channel_id = ?1
        AND items.trader_id = ?2
    ",
    )?;

    let rows = stmt.query_map(params![channel_id, user_id], |row| {
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
    let conn = Connection::open("C:/path_to_your_db/trading_bot.db")?;

    let mut stmt = conn.prepare(
        "
        SELECT trader1_paid, trader2_paid 
        FROM trades 
        WHERE channel_id = ?1 
        AND (trader1_id = ?2 OR trader2_id = ?2)
    ",
    )?;

    let mut has_paid = false;
    let mut rows = stmt.query(params![channel_id, user_id])?;

    if let Some(row) = rows.next()? {
        // Check which trader the user is (trader1 or trader2) and retrieve the corresponding paid status
        let (trader1_paid, trader2_paid): (bool, bool) = (row.get(0)?, row.get(1)?);

        // If user is trader1, check trader1_paid, else check trader2_paid
        has_paid = if row.get::<_, String>(2)? == user_id {
            trader1_paid
        } else {
            trader2_paid
        };
    }

    Ok(has_paid)
}

pub fn set_gold_fee_status(channel_id: &str, user_id: &str, has_paid: bool) -> Result<()> {
    let conn = Connection::open("C:/path_to_your_db/trading_bot.db")?;

    // First, identify whether the user is trader1 or trader2 in the trade
    let mut stmt = conn.prepare(
        "
        SELECT trader1_id, trader2_id 
        FROM trades 
        WHERE channel_id = ?1
    ",
    )?;

    let mut rows = stmt.query(params![channel_id])?;

    if let Some(row) = rows.next()? {
        let (trader1_id, trader2_id): (String, String) = (row.get(0)?, row.get(1)?);

        // Determine which trader the user is and update the corresponding paid status
        if user_id == trader1_id {
            let mut stmt = conn.prepare(
                "
                UPDATE trades 
                SET trader1_paid = ?2 
                WHERE channel_id = ?1
            ",
            )?;
            stmt.execute(params![channel_id, has_paid])?;
        } else if user_id == trader2_id {
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

pub fn get_gold_for_user(channel_id: &str, user_id: &str) -> Result<i32> {
    let conn = Connection::open("C:/path_to_your_db/trading_bot.db")?;

    let mut stmt = conn.prepare(
        "
        SELECT trader1_gold, trader2_gold, trader1_id, trader2_id 
        FROM trades 
        JOIN traders ON traders.id = trades.trader1_id OR traders.id = trades.trader2_id 
        WHERE trades.channel_id = ?1 
        AND traders.discord_id = ?2
    ",
    )?;

    let mut rows = stmt.query(params![channel_id, user_id])?;

    if let Some(row) = rows.next()? {
        let (trader1_gold, trader2_gold, trader1_id, trader2_id): (i32, i32, String, String) =
            (row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?);

        // Determine whether the user is trader1 or trader2, and return the corresponding gold amount
        return Ok(if user_id == trader1_id {
            trader1_gold
        } else if user_id == trader2_id {
            trader2_gold
        } else {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        });
    }

    Err(rusqlite::Error::QueryReturnedNoRows)
}

pub fn set_item_status_by_urls(
    item_image_url: &str,
    info_image_url: &str,
    new_status: &str,
) -> Result<()> {
    let conn = Connection::open("C:/path_to_your_db/trading_bot.db")?;

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
