use rusqlite::{params, Connection, Result};

pub fn get_links_for_user(channel_id: &str, user_id: &str) -> Result<Vec<String>> {
    let conn = Connection::open("path_to_your_database.db")?;

    let mut stmt = conn.prepare("SELECT item_link FROM trade_items WHERE channel_id = ? AND user_id = ?")?;
    let rows = stmt.query_map(params![channel_id, user_id], |row| {
        row.get(0)
    })?;

    let mut links = Vec::new();
    for link_result in rows {
        links.push(link_result?);
    }

    Ok(links)
}