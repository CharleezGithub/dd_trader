use rusqlite::{params, Connection, Result};

pub fn get_links_for_user(channel_id: &str, user_id: &str) -> Result<Vec<String>> {
    let conn = Connection::open("C:/Users/Alex/Desktop/VSCode/dd_trader/trading_bot.db")?;

    // Updated SQL query to fetch the desired data based on the provided table structures
    let mut stmt = conn.prepare("
        SELECT items.info_image_url
        FROM items
        JOIN trades ON items.trade_id = trades.id
        WHERE trades.channel_id = ?1
        AND items.trader_id = ?2
    ")?;
    let rows = stmt.query_map(params![channel_id, user_id], |row| {
        row.get(0)
    })?;

    let mut links = Vec::new();
    for link_result in rows {
        links.push(link_result?);
    }

    Ok(links)
}

fn check_gold_fee() {
    todo!();
}