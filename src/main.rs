use chrono::{NaiveDate, NaiveDateTime};
use console::style;
use dialoguer::{Confirm, Input, Select};
use rusqlite::{Connection, OptionalExtension, Result, params};
use rust_llm_astrology_agent::{api, dasha, geo, math, rules, utils};
use std::env;
use std::io::{self, Write};
use tokio::io::AsyncWriteExt;

fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS Clients (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            city TEXT NOT NULL,
            birth_data TEXT NOT NULL,
            status TEXT NOT NULL,
            dob TEXT,
            time TEXT
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Readings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            client_id INTEGER NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            question TEXT NOT NULL,
            full_ai_response TEXT NOT NULL,
            FOREIGN KEY(client_id) REFERENCES Clients(id)
        )",
        (),
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_clients_name ON Clients(name)",
        (),
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_readings_client_id ON Readings(client_id)",
        (),
    )?;
    Ok(())
}

fn ensure_client_columns(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(Clients)")?;
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    if !columns.contains(&"dob".to_string()) {
        conn.execute("ALTER TABLE Clients ADD COLUMN dob TEXT", [])?;
    }
    if !columns.contains(&"time".to_string()) {
        conn.execute("ALTER TABLE Clients ADD COLUMN time TEXT", [])?;
    }
    Ok(())
}

pub fn prompt_input(prompt: &str) -> Option<String> {
    Input::<String>::new()
        .with_prompt(prompt)
        .interact_text()
        .ok()
}

pub fn prompt_select(prompt: &str, items: &[&str], default: usize) -> Option<usize> {
    Select::new()
        .with_prompt(prompt)
        .items(items)
        .default(default)
        .interact()
        .ok()
}

pub fn prompt_confirm(prompt: &str, default: bool) -> Option<bool> {
    Confirm::new()
        .with_prompt(prompt)
        .default(default)
        .interact()
        .ok()
}

fn view_client_reading_history(conn: &Connection) -> Result<()> {
    let search_name = match prompt_input("Enter client name to view reading history") {
        Some(s) if !s.trim().is_empty() => s,
        _ => {
            println!("Action cancelled.");
            return Ok(());
        }
    };

    let mut stmt =
        conn.prepare("SELECT id, name, city, status, dob, time FROM Clients WHERE name LIKE ?")?;
    let query_term = format!("%{}%", search_name);
    let clients = stmt
        .query_map(params![query_term], ClientRecord::from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    if clients.is_empty() {
        println!("{}", style("No matching client found.").red());
        return Ok(());
    }

    let target_client = if clients.len() == 1 {
        &clients[0]
    } else {
        println!("\nMultiple clients match:");
        for c in &clients {
            println!("  [{}] {} ({})", c.id, c.name, c.city);
        }
        let id_str = match prompt_input("Enter the exact ID of the client") {
            Some(s) => s,
            None => {
                println!("Action cancelled.");
                return Ok(());
            }
        };
        let target_id = id_str.trim().parse::<i64>().unwrap_or(-1);
        match clients.iter().find(|c| c.id == target_id) {
            Some(c) => c,
            None => {
                println!("{}", style("Client ID not found.").red());
                return Ok(());
            }
        }
    };

    let readings = get_client_readings(conn, target_client.id)?;
    if readings.is_empty() {
        println!(
            "No historical readings found for {}.",
            style(&target_client.name).bold()
        );
        return Ok(());
    }

    println!(
        "\n--- Reading History for {} ({} recorded) ---",
        style(&target_client.name).cyan().bold(),
        readings.len()
    );

    for (idx, r) in readings.iter().enumerate() {
        println!(
            "\n[Reading #{}] Timestamp: {}",
            readings.len() - idx,
            r.timestamp
        );
        println!("Question: {}", style(&r.question).yellow());
        println!("Response:\n{}", r.full_ai_response);
        println!("{}", "-".repeat(60));
    }

    Ok(())
}

fn init_db() -> Result<Connection> {
    let conn = Connection::open("astrology_journal.db")?;
    create_tables(&conn)?;
    ensure_client_columns(&conn)?;
    Ok(conn)
}

fn manage_client(
    conn: &mut Connection,
    name: &str,
    city: &str,
    dob: &str,
    time: &str,
    birth_data: &str,
) -> Result<(i64, String)> {
    let tx = conn.transaction()?;

    let client_opt = {
        let mut stmt = tx.prepare("SELECT id, status FROM Clients WHERE name = ?")?;
        stmt.query_row(params![name], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .optional()?
    };

    let res = match client_opt {
        Some((id, status)) => {
            tx.execute(
                "UPDATE Clients SET city = ?, birth_data = ?, dob = ?, time = ? WHERE id = ?",
                params![city, birth_data, dob, time, id],
            )?;
            println!(
                "{}",
                style(format!(
                    "\n[!] ✨ Repeat Customer Detected: Welcome back, {} (ID: {})",
                    name, id
                ))
                .cyan()
                .bold()
            );
            (id, status)
        }
        None => {
            tx.execute(
                "INSERT INTO Clients (name, city, birth_data, status, dob, time) VALUES (?, ?, ?, ?, ?, ?)",
                params![name, city, birth_data, "Active", dob, time],
            )?;
            let id = tx.last_insert_rowid();
            println!(
                "{}",
                style(format!("\n[+] 🆕 New Client Profile Created: {}", name))
                    .green()
                    .bold()
            );
            (id, "Active".to_string())
        }
    };

    tx.commit()?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_test_db() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        create_tables(&conn)?;
        Ok(conn)
    }

    #[test]
    fn test_manage_client_new_and_repeat() -> Result<()> {
        let mut conn = init_test_db()?;

        // New client creation
        let (id1, status1) = manage_client(
            &mut conn,
            "John Doe",
            "London",
            "15/08/1990",
            "10:45 AM",
            "Date: 15/08/1990, Time: 10:45 AM, UTC Offset: 1.00",
        )?;
        assert_eq!(id1, 1);
        assert_eq!(status1, "Active");

        // Repeat client check with updated city and details
        let (id2, status2) = manage_client(
            &mut conn,
            "John Doe",
            "Manchester",
            "15/08/1990",
            "11:00 AM",
            "Date: 15/08/1990, Time: 11:00 AM, UTC Offset: 1.00",
        )?;
        assert_eq!(id2, 1);
        assert_eq!(status2, "Active");

        // Verify that city and birth_data were updated in database
        let mut stmt = conn.prepare("SELECT city, birth_data, time FROM Clients WHERE id = ?")?;
        let (city, birth_data, time): (String, String, String) =
            stmt.query_row(params![id2], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
        assert_eq!(city, "Manchester");
        assert_eq!(time, "11:00 AM");
        assert_eq!(
            birth_data,
            "Date: 15/08/1990, Time: 11:00 AM, UTC Offset: 1.00"
        );

        Ok(())
    }

    #[test]
    fn test_update_non_existent_client_returns_zero_affected_rows() -> Result<()> {
        let conn = init_test_db()?;

        let non_existent_id = 99999;
        let rows = conn.execute(
            "UPDATE Clients SET name = ? WHERE id = ?",
            params!["Non Existent", non_existent_id],
        )?;

        assert_eq!(rows, 0);
        Ok(())
    }

    #[test]
    fn test_update_client_dob_and_time() -> Result<()> {
        let mut conn = init_test_db()?;

        let (client_id, _) = manage_client(
            &mut conn,
            "Eve Adams",
            "Chicago",
            "01/01/1990",
            "09:00 AM",
            "Date: 01/01/1990, Time: 09:00 AM, UTC Offset: -6.00",
        )?;

        conn.execute(
            "UPDATE Clients SET dob = ?, time = ? WHERE id = ?",
            params!["02/02/1991", "10:30 AM", client_id],
        )?;

        let mut stmt =
            conn.prepare("SELECT id, name, city, status, dob, time FROM Clients WHERE id = ?")?;
        let client = stmt.query_row(params![client_id], ClientRecord::from_row)?;

        assert_eq!(client.dob, Some("02/02/1991".to_string()));
        assert_eq!(client.time, Some("10:30 AM".to_string()));

        Ok(())
    }

    #[test]
    fn test_save_reading_and_query() -> Result<()> {
        let mut conn = init_test_db()?;

        let (client_id, _) = manage_client(
            &mut conn,
            "Jane Smith",
            "Paris",
            "01/01/1995",
            "14:30",
            "Date: 01/01/1995, Time: 14:30, UTC Offset: 1.00",
        )?;

        save_reading(
            &conn,
            client_id,
            "What is my career outlook?",
            "Promising alignment.",
        )?;

        let mut stmt = conn.prepare(
            "SELECT client_id, question, full_ai_response FROM Readings WHERE client_id = ?",
        )?;
        let reading_row = stmt.query_row(params![client_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        assert_eq!(reading_row.0, client_id);
        assert_eq!(reading_row.1, "What is my career outlook?");
        assert_eq!(reading_row.2, "Promising alignment.");

        Ok(())
    }

    #[test]
    fn test_delete_client_transaction() -> Result<()> {
        let mut conn = init_test_db()?;

        let (client_id, _) = manage_client(
            &mut conn,
            "Alice Smith",
            "Paris",
            "01/01/1995",
            "14:30",
            "Date: 01/01/1995, Time: 14:30, UTC Offset: 1.00",
        )?;

        save_reading(
            &conn,
            client_id,
            "What is my future?",
            "Bright future ahead.",
        )?;

        let count_clients: i64 = conn.query_row(
            "SELECT COUNT(*) FROM Clients WHERE id = ?",
            params![client_id],
            |r| r.get(0),
        )?;
        assert_eq!(count_clients, 1);

        let count_readings: i64 = conn.query_row(
            "SELECT COUNT(*) FROM Readings WHERE client_id = ?",
            params![client_id],
            |r| r.get(0),
        )?;
        assert_eq!(count_readings, 1);

        delete_client_record(&mut conn, client_id)?;

        let count_clients_after: i64 = conn.query_row(
            "SELECT COUNT(*) FROM Clients WHERE id = ?",
            params![client_id],
            |r| r.get(0),
        )?;
        assert_eq!(count_clients_after, 0);

        let count_readings_after: i64 = conn.query_row(
            "SELECT COUNT(*) FROM Readings WHERE client_id = ?",
            params![client_id],
            |r| r.get(0),
        )?;
        assert_eq!(count_readings_after, 0);

        Ok(())
    }

    #[test]
    fn test_search_clients_query() -> Result<()> {
        let mut conn = init_test_db()?;

        manage_client(
            &mut conn,
            "Alice Smith",
            "Paris",
            "01/01/1995",
            "14:30",
            "Date: 01/01/1995, Time: 14:30, UTC Offset: 1.00",
        )?;
        manage_client(
            &mut conn,
            "Bob Jones",
            "Berlin",
            "02/02/1992",
            "11:15",
            "Date: 02/02/1992, Time: 11:15, UTC Offset: 1.00",
        )?;

        let search_term = "%Smith%";
        let mut stmt = conn
            .prepare("SELECT id, name, city, status, dob, time FROM Clients WHERE name LIKE ?")?;
        let results: Vec<ClientRecord> = stmt
            .query_map(params![search_term], ClientRecord::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Alice Smith");
        assert_eq!(results[0].city, "Paris");

        Ok(())
    }

    #[test]
    fn test_update_client_status_and_city() -> Result<()> {
        let mut conn = init_test_db()?;

        let (client_id, _) = manage_client(
            &mut conn,
            "Charlie Brown",
            "Tokyo",
            "10/10/1988",
            "08:00 AM",
            "Date: 10/10/1988, Time: 08:00 AM, UTC Offset: 9.00",
        )?;

        conn.execute(
            "UPDATE Clients SET city = ?, status = ? WHERE id = ?",
            params!["Osaka", "Refused", client_id],
        )?;

        let mut stmt =
            conn.prepare("SELECT id, name, city, status, dob, time FROM Clients WHERE id = ?")?;
        let client = stmt.query_row(params![client_id], ClientRecord::from_row)?;

        assert_eq!(client.city, "Osaka");
        assert_eq!(client.status, "Refused");

        Ok(())
    }

    #[test]
    fn test_ensure_client_columns_legacy_schema() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        // Create legacy schema without dob and time
        conn.execute(
            "CREATE TABLE Clients (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                city TEXT NOT NULL,
                birth_data TEXT NOT NULL,
                status TEXT NOT NULL
            )",
            (),
        )?;

        ensure_client_columns(&conn)?;

        let mut stmt = conn.prepare("PRAGMA table_info(Clients)")?;
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?;

        assert!(columns.contains(&"dob".to_string()));
        assert!(columns.contains(&"time".to_string()));

        // Calling it again should be idempotent and not error out
        assert!(ensure_client_columns(&conn).is_ok());

        Ok(())
    }
}

fn save_reading(conn: &Connection, client_id: i64, question: &str, response: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO Readings (client_id, question, full_ai_response) VALUES (?, ?, ?)",
        params![client_id, question, response],
    )?;
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub struct ReadingRecord {
    pub id: i64,
    pub client_id: i64,
    pub timestamp: String,
    pub question: String,
    pub full_ai_response: String,
}

impl ReadingRecord {
    pub fn from_row(row: &rusqlite::Row<'_>) -> Result<Self> {
        Ok(ReadingRecord {
            id: row.get(0)?,
            client_id: row.get(1)?,
            timestamp: row.get(2)?,
            question: row.get(3)?,
            full_ai_response: row.get(4)?,
        })
    }
}

pub fn get_client_readings(conn: &Connection, client_id: i64) -> Result<Vec<ReadingRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, client_id, timestamp, question, full_ai_response FROM Readings WHERE client_id = ? ORDER BY timestamp DESC",
    )?;
    let readings = stmt
        .query_map(params![client_id], ReadingRecord::from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(readings)
}

#[derive(Debug, PartialEq, Eq)]
pub struct ClientRecord {
    pub id: i64,
    pub name: String,
    pub city: String,
    pub status: String,
    pub dob: Option<String>,
    pub time: Option<String>,
}

impl ClientRecord {
    pub fn from_row(row: &rusqlite::Row<'_>) -> Result<Self> {
        Ok(ClientRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            city: row.get(2)?,
            status: row.get(3)?,
            dob: row.get(4)?,
            time: row.get(5)?,
        })
    }
}

fn view_clients(conn: &Connection, results: Option<Vec<ClientRecord>>) -> Result<()> {
    let list = match results {
        Some(r) => r,
        None => {
            let mut stmt = conn.prepare("SELECT id, name, city, status, dob, time FROM Clients")?;
            stmt.query_map([], ClientRecord::from_row)?
                .collect::<Result<Vec<_>, _>>()?
        }
    };

    if list.is_empty() {
        println!("No records found.");
        return Ok(());
    }

    println!(
        "\n{:<4} | {:<20} | {:<12} | {:<10} | {:<10} | {:<8}",
        "ID", "Name", "City", "Status", "DOB", "Time"
    );
    println!("{}", "-".repeat(75));

    for client in list {
        let dob_disp = client.dob.unwrap_or_else(|| "N/A".to_string());
        let time_disp = client.time.unwrap_or_else(|| "N/A".to_string());
        println!(
            "{:<4} | {:<20} | {:<12} | {:<10} | {:<10} | {:<8}",
            client.id, client.name, client.city, client.status, dob_disp, time_disp
        );
    }
    println!();
    Ok(())
}

fn search_clients(conn: &Connection) -> Result<()> {
    let search_term = match prompt_input("Enter part or all of the client's name") {
        Some(s) if !s.trim().is_empty() => s,
        _ => {
            println!("Action cancelled.");
            return Ok(());
        }
    };

    let mut stmt =
        conn.prepare("SELECT id, name, city, status, dob, time FROM Clients WHERE name LIKE ?")?;
    let query_term = format!("%{}%", search_term);
    let results = stmt
        .query_map(params![query_term], ClientRecord::from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    if results.is_empty() {
        println!("{}", style("No matching clients found.").red());
    } else {
        println!("\n--- Search Results for '{}' ---", search_term);
        view_clients(conn, Some(results))?;
    }
    Ok(())
}

fn edit_client(conn: &Connection) -> Result<()> {
    let id_str = match prompt_input("Enter the ID of the client (or type 0 to cancel)") {
        Some(s) => s,
        None => return Ok(()),
    };

    let id: i64 = id_str.trim().parse().unwrap_or(0);
    if id == 0 {
        println!("Action cancelled.");
        return Ok(());
    }

    let fields = &[
        "Name",
        "City",
        "Date of Birth (DOB)",
        "Time of Birth",
        "Status (Active/Refused)",
    ];
    let selection = match prompt_select("What would you like to update?", fields, 0) {
        Some(s) => s,
        None => return Ok(()),
    };

    match selection {
        0 => {
            let new_name = match prompt_input("Enter new Name") {
                Some(s) => s,
                None => return Ok(()),
            };
            let rows = conn.execute(
                "UPDATE Clients SET name = ? WHERE id = ?",
                params![new_name, id],
            )?;
            if rows == 0 {
                println!(
                    "{}",
                    style(format!("No client found with ID {}.", id)).red()
                );
            } else {
                println!("Client Name updated successfully.");
            }
        }
        1 => {
            let new_city = match prompt_input("Enter new City") {
                Some(s) => s,
                None => return Ok(()),
            };
            let rows = conn.execute(
                "UPDATE Clients SET city = ? WHERE id = ?",
                params![new_city, id],
            )?;
            if rows == 0 {
                println!(
                    "{}",
                    style(format!("No client found with ID {}.", id)).red()
                );
            } else {
                println!("Client City updated successfully.");
            }
        }
        2 => {
            let new_dob = loop {
                let input = match prompt_input(
                    "Enter new Date of Birth (e.g., 15/08/1990 or 1990-08-15)",
                ) {
                    Some(s) => s,
                    None => return Ok(()),
                };
                if utils::parse_flexible_date(&input).is_some() {
                    break input;
                }
                println!(
                    "{}",
                    style("Invalid date format. Please use DD/MM/YYYY or YYYY-MM-DD (e.g., 15/08/1990).")
                        .red()
                );
            };
            let rows = conn.execute(
                "UPDATE Clients SET dob = ? WHERE id = ?",
                params![new_dob, id],
            )?;
            if rows == 0 {
                println!(
                    "{}",
                    style(format!("No client found with ID {}.", id)).red()
                );
            } else {
                println!("Client DOB updated successfully.");
            }
        }
        3 => {
            let new_time = loop {
                let input = match prompt_input("Enter new Time of Birth (e.g., 10:45 AM or 14:30)")
                {
                    Some(s) => s,
                    None => return Ok(()),
                };
                if utils::parse_flexible_time(&input).is_some() {
                    break input;
                }
                println!(
                    "{}",
                    style(
                        "Invalid time format. Please use HH:MM AM/PM or HH:MM (e.g., 10:45 AM or 14:30)."
                    )
                    .red()
                );
            };
            let rows = conn.execute(
                "UPDATE Clients SET time = ? WHERE id = ?",
                params![new_time, id],
            )?;
            if rows == 0 {
                println!(
                    "{}",
                    style(format!("No client found with ID {}.", id)).red()
                );
            } else {
                println!("Client Time of Birth updated successfully.");
            }
        }
        4 => {
            let status_options = &["Active", "Refused"];
            let status_selection = match prompt_select("Select new Status", status_options, 0) {
                Some(s) => s,
                None => return Ok(()),
            };
            let new_status = status_options[status_selection];
            let rows = conn.execute(
                "UPDATE Clients SET status = ? WHERE id = ?",
                params![new_status, id],
            )?;
            if rows == 0 {
                println!(
                    "{}",
                    style(format!("No client found with ID {}.", id)).red()
                );
            } else {
                println!("Client Status updated to '{}' successfully.", new_status);
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn delete_client_record(conn: &mut Connection, id: i64) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM Readings WHERE client_id = ?", params![id])?;
    tx.execute("DELETE FROM Clients WHERE id = ?", params![id])?;
    tx.commit()?;
    Ok(())
}

fn delete_client(conn: &mut Connection) -> Result<()> {
    let id_str = match prompt_input("Enter the ID of the client to DELETE (or 0 to cancel)") {
        Some(s) => s,
        None => return Ok(()),
    };

    let id: i64 = id_str.trim().parse().unwrap_or(0);
    if id == 0 {
        println!("Action cancelled.");
        return Ok(());
    }

    let confirmed = prompt_confirm(
        "Are you sure you want to delete this client and all their readings? This cannot be undone.",
        false,
    )
    .unwrap_or(false);

    if confirmed {
        delete_client_record(conn, id)?;
        println!("Client and associated records deleted permanently.");
    } else {
        println!("Deletion cancelled.");
    }
    Ok(())
}

pub struct ReadingParams {
    pub name: String,
    pub date: String,
    pub time: String,
    pub city: String,
    pub question: String,
    pub target_words: u32,
}

fn launch_wizard() -> Option<ReadingParams> {
    println!("\n--- Run New Astrology Reading ---");

    let name = prompt_input("Enter Client Name")?;

    let date = loop {
        let input = prompt_input("Enter Date of Birth (e.g., 15/08/1990 or 1990-08-15)")?;
        if utils::parse_flexible_date(&input).is_some() {
            break input;
        }
        println!(
            "{}",
            style("Invalid date format. Please use DD/MM/YYYY or YYYY-MM-DD (e.g., 15/08/1990).")
                .red()
        );
    };

    let time = loop {
        let input = prompt_input("Enter Time of Birth (e.g., 10:45 AM or 14:30)")?;
        if utils::parse_flexible_time(&input).is_some() {
            break input;
        }
        println!(
            "{}",
            style(
                "Invalid time format. Please use HH:MM AM/PM or HH:MM (e.g., 10:45 AM or 14:30)."
            )
            .red()
        );
    };

    let city = prompt_input("Enter City of Birth")?;
    let question = prompt_input("Enter the Querent's Question")?;

    let target_words = prompt_target_words();

    Some(ReadingParams {
        name,
        date,
        time,
        city,
        question,
        target_words,
    })
}

fn prompt_target_words() -> u32 {
    let words_str = prompt_input("Enter desired reading length in words (e.g., 500)")
        .unwrap_or_else(|| "500".to_string());
    words_str.trim().parse().unwrap_or(500)
}

fn fast_track_reading(conn: &Connection) -> Result<Option<ReadingParams>> {
    println!("\n--- Fast-Track Existing Client ---");
    let search_name = match prompt_input("Enter the Name of the client (or type 'cancel' to exit)")
    {
        Some(s) => s,
        None => return Ok(None),
    };

    if search_name.trim().eq_ignore_ascii_case("cancel") {
        println!("Action cancelled.");
        return Ok(None);
    }

    let mut stmt =
        conn.prepare("SELECT id, name, city, status, dob, time FROM Clients WHERE name LIKE ?")?;
    let query_term = format!("%{}%", search_name);
    let results: Vec<ClientRecord> = stmt
        .query_map(params![query_term], ClientRecord::from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    if results.is_empty() {
        println!("Client not found.");
        return Ok(None);
    }

    let target_id = if results.len() == 1 {
        results[0].id
    } else {
        for client in &results {
            let dob_disp = client.dob.as_deref().unwrap_or("N/A");
            let time_disp = client.time.as_deref().unwrap_or("N/A");
            println!(
                "[{}] {} - DOB: {} | Time: {} | Place: {}",
                client.id, client.name, dob_disp, time_disp, client.city
            );
        }

        let selected_id_str = match prompt_input(
            "Enter the specific ID of the correct match from this detailed list",
        ) {
            Some(s) => s,
            None => return Ok(None),
        };

        match selected_id_str.trim().parse::<i64>() {
            Ok(id) => id,
            Err(_) => {
                println!("Invalid ID format.");
                return Ok(None);
            }
        }
    };

    let client_opt = results.into_iter().find(|r| r.id == target_id);

    if let Some(client) = client_opt {
        if client.status == "Refused" {
            println!(
                "{}",
                style("This client is marked as 'Refused'. Fast-Track denied.")
                    .red()
                    .bold()
            );
            return Ok(None);
        }

        let dob = client.dob.unwrap_or_default();
        let time = client.time.unwrap_or_default();

        if dob.is_empty() || time.is_empty() {
            println!("{}", style("Incomplete Profile: Missing DOB or Time. Please use the New Reading wizard for this client.").red());
            return Ok(None);
        }

        println!(
            "{}",
            style(format!("[+] 🚀 Fast-Tracking Reading for: {}", client.name))
                .green()
                .bold()
        );

        let question = match prompt_input("Enter the Querent's NEW Question") {
            Some(q) => q,
            None => return Ok(None),
        };

        let target_words = prompt_target_words();

        Ok(Some(ReadingParams {
            name: client.name,
            date: dob,
            time,
            city: client.city,
            question,
            target_words,
        }))
    } else {
        println!("Client ID not found.");
        Ok(None)
    }
}

async fn execute_reading_flow(
    conn: &mut Connection,
    name: String,
    date_str: String,
    time_str: String,
    city: String,
    question: String,
    target_words: u32,
) {
    let client = reqwest::Client::builder()
        .connection_verbose(false)
        .tcp_keepalive(None)
        .pool_idle_timeout(std::time::Duration::from_secs(5))
        .pool_max_idle_per_host(1)
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .expect("Network Initialization Error");

    // Prepare NaiveDateTime for location/offset resolution
    let date = match utils::parse_flexible_date(&date_str) {
        Some(d) => d,
        None => {
            eprintln!("Invalid Date Format. Please use DD/MM/YYYY or YYYY-MM-DD.");
            return;
        }
    };
    let time = match utils::parse_flexible_time(&time_str) {
        Some(t) => t,
        None => {
            eprintln!("Invalid Time Format. Please use HH:MM AM/PM or HH:MM.");
            return;
        }
    };
    let naive_dt = NaiveDateTime::new(date, time);

    println!("\nInitializing Astrology Workflow...");

    println!("Resolving Location and Historical Timezone for {}...", city);
    let loc_data = match geo::get_location_data(&client, &city, naive_dt).await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Location Resolution Error: {}", e);
            return;
        }
    };

    let (lat, lon, offset) = (
        loc_data.latitude,
        loc_data.longitude,
        loc_data.utc_offset_hours,
    );

    let birth_data_summary = format!(
        "Date: {}, Time: {}, UTC Offset: {:.2}",
        date_str, time_str, offset
    );

    println!("Managing Client Record for {}...", name);
    let (client_id, status) = match manage_client(
        conn,
        &name,
        &city,
        &date_str,
        &time_str,
        &birth_data_summary,
    ) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Database Error: {}", e);
            return;
        }
    };

    if status == "Refused" {
        println!(
            "{}",
            style(format!(
                "Access Denied: The client {} is marked as 'Refused'.",
                name
            ))
            .red()
            .bold()
        );
        return;
    }

    println!("\nConfigure Chart Parameters:");
    let house_options = &[
        "Placidus (Required for KP Astrology)",
        "Whole Sign (Required for Standard Vedic)",
    ];

    let house_selection = prompt_select("Select House System", house_options, 1).unwrap_or(1);

    let selected_house_system = match house_selection {
        0 => math::HouseSystem::Placidus,
        1 => math::HouseSystem::WholeSign,
        _ => math::HouseSystem::WholeSign,
    };

    println!("\nCalculating Chart for {}...", name);
    let birth_details = math::BirthDetails {
        date: date_str.clone(),
        time: time_str.clone(),
        latitude: lat,
        longitude: lon,
        timezone: offset,
        system: math::System::Vedic,
        house_system: selected_house_system,
    };

    let (chart_summary, moon_lon, parivartan_alerts) =
        match math::calculate_astrology(birth_details) {
            Ok(astro_data) => {
                let moon_lon = astro_data
                    .planets
                    .iter()
                    .find(|p| p.name == "Moon")
                    .map(|p| p.longitude)
                    .unwrap_or(0.0);
                let expert_data = rules::process(&astro_data);
                let parivartan = math::detect_parivartan_yogas(&astro_data.planets);
                (rules::format_summary(&expert_data), moon_lon, parivartan)
            }
            Err(e) => {
                eprintln!("Math Layer Error: {}", e);
                return;
            }
        };

    println!("Extracting Target Timeframe via Agent 1...");
    let current_date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let extracted_target_date_str =
        crate::api::extract_target_date(&client, &question, &current_date).await;

    println!("[*] Agent 1 Complete. Initiating 31-second API cooldown to prevent rate-limiting...");
    tokio::time::sleep(std::time::Duration::from_secs(31)).await;

    let target_date = NaiveDate::parse_from_str(&extracted_target_date_str, "%Y-%m-%d")
        .unwrap_or_else(|_| {
            chrono::Local::now()
                .naive_local()
                .date()
                .checked_add_signed(chrono::Duration::try_days(365).unwrap_or_default())
                .unwrap()
        });
    let start_date_now = chrono::Local::now().naive_local().date();

    println!("Calculating Vimshottari Dasha Timeline...");
    let dasha_timeline =
        crate::dasha::generate_dasha_timeline(moon_lon, naive_dt, start_date_now, target_date);

    let final_chart_summary = format!(
        "{}\n\n--- ACTIVE VIMSHOTTARI DASHA TIMELINE (From Today to Target Date) ---\n{}\n",
        chart_summary, dasha_timeline
    );

    println!("Orchestrating AI Prompt for {}...", name);
    let current_date = chrono::Local::now().format("%d %B %Y").to_string();
    // [PROPRIETARY ASTROLOGICAL MASTER PROMPT REDACTED FOR PUBLIC REPOSITORY]
    let system_prompt = format!(
        "[PROPRIETARY ASTROLOGICAL MASTER PROMPT REDACTED FOR PUBLIC REPOSITORY]\n\
    Today's date: {}. Target word count: {}.",
        current_date, target_words
    );

    let user_prompt = format!(
        "Querent: {}\nQuestion: {}\nResolution: City: {}, Lat: {}, Lon: {}, Offset: {}\n\nChart Data:\n{}",
        name, question, city, lat, lon, offset, final_chart_summary
    );

    // Data Privacy Layer
    let mut anonymized_user_prompt = user_prompt.replace(&name, "The Querent");

    if !parivartan_alerts.is_empty() {
        anonymized_user_prompt.push_str(&parivartan_alerts);
    }

    let combined_prompt = format!("{}\n\n{}", system_prompt, anonymized_user_prompt);
    tokio::fs::create_dir_all("readings")
        .await
        .unwrap_or_default();
    let _ = tokio::fs::write("readings/last_prompt_log.txt", &combined_prompt).await;

    println!("Calling Gemini API...");
    let model = api::get_gemini_model();
    let final_reading = match api::call_gemini_with_retry(
        &client,
        system_prompt.clone(),
        anonymized_user_prompt.clone(),
        &model,
        2000,
    )
    .await
    {
        Ok(reading) => reading,
        Err(e) => {
            eprintln!(
                "\n{}",
                style(format!("[!] System Latency: {:?}", e)).red().bold()
            );
            return;
        }
    };

    if let Err(e) = save_reading(conn, client_id, &question, &final_reading) {
        eprintln!("Failed to save reading: {}", e);
    } else {
        println!("Reading successfully archived.");
    }

    // Presentation Layer
    tokio::fs::create_dir_all("readings")
        .await
        .unwrap_or_default();

    let html_content = utils::generate_html_report(&name, &final_reading);

    let clean_name = utils::sanitize_filename(&name);
    let date_suffix = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_{}.html", clean_name, date_suffix);

    let mut absolute_path = std::env::current_dir().unwrap();
    absolute_path.push("readings");
    absolute_path.push(&filename);

    if let Ok(mut file) = tokio::fs::File::create(&absolute_path).await {
        if let Err(e) = file.write_all(html_content.as_bytes()).await {
            eprintln!("Failed to write HTML report to file: {}", e);
        } else {
            println!(
                "Reading generated! Opening in browser at: {}",
                absolute_path.display()
            );
            if let Err(e) = open::that(&absolute_path) {
                println!(
                    "{}",
                    style(format!(
                        "[!] Warning: Could not open HTML report automatically ({}) at {}",
                        e,
                        absolute_path.display()
                    ))
                    .yellow()
                );
            }
        }
    } else {
        // Fallback to terminal
        println!(
            "\n--- AI Vedic Reading for {} ---\n{}\n--- End of Reading ---",
            name, final_reading
        );
    }
}

fn wait_for_enter() {
    print!("\nPress Enter to return to Main Menu...");
    io::stdout().flush().unwrap();
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
}

#[tokio::main]
async fn main() {
    let mut conn = init_db().expect("Database Initialization Error");
    let args: Vec<String> = env::args().collect();

    // Support CLI mode for backwards compatibility
    if args.len() >= 6 {
        let name = args[1].clone();
        let date_str = args[2].clone();
        let time_str = args[3].clone();
        let city = args[4].clone();
        let question = args[5].clone();
        let target_words = if args.len() >= 7 {
            args[6].parse().unwrap_or(500)
        } else {
            500
        };
        execute_reading_flow(
            &mut conn,
            name,
            date_str,
            time_str,
            city,
            question,
            target_words,
        )
        .await;
        return;
    }

    loop {
        let menu_options = &[
            "🔮 Run a New Astrology Reading",
            "🔄 Run Reading for Existing Client",
            "📜 View All Client Records",
            "📖 View Client Reading History",
            "🔍 Search Client by Name",
            "✏️ Edit a Client's Details",
            "❌ Delete a Client",
            "🚪 Exit Program",
        ];

        let selection = match prompt_select("Main Menu - Select an Action", menu_options, 0) {
            Some(idx) => idx,
            None => {
                println!("\nExiting Program... Goodbye!");
                break;
            }
        };

        match selection {
            0 => {
                if let Some(params) = launch_wizard() {
                    execute_reading_flow(
                        &mut conn,
                        params.name,
                        params.date,
                        params.time,
                        params.city,
                        params.question,
                        params.target_words,
                    )
                    .await;
                }
                wait_for_enter();
            }
            1 => {
                if let Ok(Some(params)) = fast_track_reading(&conn) {
                    execute_reading_flow(
                        &mut conn,
                        params.name,
                        params.date,
                        params.time,
                        params.city,
                        params.question,
                        params.target_words,
                    )
                    .await;
                }
                wait_for_enter();
            }
            2 => {
                let _ = view_clients(&conn, None);
                wait_for_enter();
            }
            3 => {
                let _ = view_client_reading_history(&conn);
                wait_for_enter();
            }
            4 => {
                let _ = search_clients(&conn);
                wait_for_enter();
            }
            5 => {
                let _ = edit_client(&conn);
                wait_for_enter();
            }
            6 => {
                let _ = delete_client(&mut conn);
                wait_for_enter();
            }
            7 => {
                println!("Exiting Program... Goodbye!");
                break;
            }
            _ => unreachable!(),
        }
    }
}
