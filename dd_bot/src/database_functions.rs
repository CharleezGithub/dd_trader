use rusqlite::{params, Connection, Result};

pub fn get_links_for_user(channel_id: &str, user_id: &str) -> Result<(Vec<String>, Vec<String>)> {
    let conn = Connection::open("C:/Users/Alex/Desktop/VSCode/dd_trader/trading_bot.db")?;

    let mut stmt = conn.prepare("
        SELECT items.item_image_url, items.info_image_url
        FROM items
        JOIN trades ON items.trade_id = trades.id
        WHERE trades.channel_id = ?1
        AND items.trader_id = ?2
    ")?;
    
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
    
    let mut stmt = conn.prepare("
        SELECT trader1_paid, trader2_paid 
        FROM trades 
        WHERE channel_id = ?1 
        AND (trader1_id = ?2 OR trader2_id = ?2)
    ")?;
    
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