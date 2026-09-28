use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let db_path = "astrology_journal.db";
    if !std::path::Path::new(db_path).exists() {
        println!("Database file '{}' does not exist yet.", db_path);
        return Ok(());
    }

    let conn = Connection::open(db_path)?;

    println!("===========================================");
    println!("        ASTROLOGY JOURNAL DATABASE        ");
    println!("===========================================");

    let total_clients: i64 = conn.query_row("SELECT COUNT(*) FROM Clients", [], |r| r.get(0))?;
    let total_readings: i64 = conn.query_row("SELECT COUNT(*) FROM Readings", [], |r| r.get(0))?;

    println!(
        "Summary: {} Clients | {} Readings\n",
        total_clients, total_readings
    );

    println!("--- Registered Client Profiles ---");

    let mut stmt = conn.prepare(
        "SELECT id, name, city, status, COALESCE(dob, 'N/A'), COALESCE(time, 'N/A') FROM Clients",
    )?;
    let client_rows = stmt.query_map([], |row| {
        Ok(ClientView {
            id: row.get(0)?,
            name: row.get(1)?,
            city: row.get(2)?,
            status: row.get(3)?,
            dob: row.get(4)?,
            time: row.get(5)?,
        })
    })?;

    println!(
        "{:<4} | {:<20} | {:<12} | {:<10} | {:<10} | {:<8}",
        "ID", "Name", "City", "Status", "DOB", "Time"
    );
    println!("{}", "-".repeat(75));

    for client_res in client_rows {
        let client = client_res?;
        println!(
            "{:<4} | {:<20} | {:<12} | {:<10} | {:<10} | {:<8}",
            client.id, client.name, client.city, client.status, client.dob, client.time
        );
    }

    println!("\n--- Archived AI Readings ---");
    let mut stmt = conn.prepare(
        "SELECT r.id, c.name, r.timestamp, r.question FROM Readings r JOIN Clients c ON r.client_id = c.id ORDER BY r.timestamp DESC",
    )?;
    let reading_rows = stmt.query_map([], |row| {
        Ok(ReadingView {
            id: row.get(0)?,
            name: row.get(1)?,
            timestamp: row.get(2)?,
            question: row.get(3)?,
        })
    })?;

    for reading_res in reading_rows {
        let reading = reading_res?;
        println!(
            "Reading #{} | Client: {} | Date: {}\n  Question: {}\n",
            reading.id, reading.name, reading.timestamp, reading.question
        );
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub struct ClientView {
    pub id: i64,
    pub name: String,
    pub city: String,
    pub status: String,
    pub dob: String,
    pub time: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ReadingView {
    pub id: i64,
    pub name: String,
    pub timestamp: String,
    pub question: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_view_struct() {
        let client = ClientView {
            id: 1,
            name: "John Doe".to_string(),
            city: "London".to_string(),
            status: "Active".to_string(),
            dob: "15/08/1990".to_string(),
            time: "10:45 AM".to_string(),
        };
        assert_eq!(client.id, 1);
        assert_eq!(client.name, "John Doe");
        assert_eq!(client.city, "London");
        assert_eq!(client.status, "Active");
        assert_eq!(client.dob, "15/08/1990");
        assert_eq!(client.time, "10:45 AM");
    }

    #[test]
    fn test_reading_view_struct() {
        let reading = ReadingView {
            id: 42,
            name: "Jane Smith".to_string(),
            timestamp: "2025-01-01 12:00:00".to_string(),
            question: "Career prospects?".to_string(),
        };
        assert_eq!(reading.id, 42);
        assert_eq!(reading.name, "Jane Smith");
        assert_eq!(reading.timestamp, "2025-01-01 12:00:00");
        assert_eq!(reading.question, "Career prospects?");
    }
}
