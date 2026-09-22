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
    struct ClientView {
        id: i64,
        name: String,
        city: String,
        status: String,
        dob: String,
        time: String,
    }

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
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;

    for reading in reading_rows {
        let (id, name, timestamp, question) = reading?;
        println!(
            "Reading #{} | Client: {} | Date: {}\n  Question: {}\n",
            id, name, timestamp, question
        );
    }

    Ok(())
}
