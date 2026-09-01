use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use console::style;
use dialoguer::{Confirm, Input, Select};
use rusqlite::{Connection, OptionalExtension, Result, params};
use rust_llm_astrology_agent::{api, dasha, geo, math, rules, utils};
use std::env;
use std::io::{self, Write};
use tokio::io::AsyncWriteExt;

fn init_db() -> Result<Connection> {
    let conn = Connection::open("astrology_journal.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Clients (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            city TEXT NOT NULL,
            birth_data TEXT NOT NULL,
            status TEXT NOT NULL
        )",
        (),
    )?;

    // Non-destructive migrations
    let _ = conn.execute("ALTER TABLE Clients ADD COLUMN dob TEXT", []);
    let _ = conn.execute("ALTER TABLE Clients ADD COLUMN time TEXT", []);

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
    Ok(conn)
}

fn manage_client(
    conn: &Connection,
    name: &str,
    city: &str,
    dob: &str,
    time: &str,
    birth_data: &str,
) -> Result<(i64, String)> {
    let mut stmt = conn.prepare("SELECT id, status FROM Clients WHERE name = ?")?;
    let client_opt = stmt
        .query_row(params![name], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .optional()?;

    match client_opt {
        Some((id, status)) => {
            let _ = conn.execute(
                "UPDATE Clients SET dob = ?, time = ? WHERE id = ?",
                params![dob, time, id],
            );
            println!(
                "{}",
                style(format!(
                    "\n[!] ✨ Repeat Customer Detected: Welcome back, {} (ID: {})",
                    name, id
                ))
                .cyan()
                .bold()
            );
            Ok((id, status))
        }
        None => {
            conn.execute(
                "INSERT INTO Clients (name, city, birth_data, status, dob, time) VALUES (?, ?, ?, ?, ?, ?)",
                params![name, city, birth_data, "Active", dob, time],
            )?;
            let id = conn.last_insert_rowid();
            println!(
                "{}",
                style(format!("\n[+] 🆕 New Client Profile Created: {}", name))
                    .green()
                    .bold()
            );
            Ok((id, "Active".to_string()))
        }
    }
}

fn save_reading(conn: &Connection, client_id: i64, question: &str, response: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO Readings (client_id, question, full_ai_response) VALUES (?, ?, ?)",
        params![client_id, question, response],
    )?;
    Ok(())
}

pub struct ClientRecord {
    pub id: i64,
    pub name: String,
    pub city: String,
    pub status: String,
    pub dob: Option<String>,
    pub time: Option<String>,
}

fn view_clients(conn: &Connection, results: Option<Vec<ClientRecord>>) -> Result<()> {
    let list = match results {
        Some(r) => r,
        None => {
            let mut stmt = conn.prepare("SELECT id, name, city, status, dob, time FROM Clients")?;
            stmt.query_map([], |row| {
                Ok(ClientRecord {
                    id: row.get::<_, i64>(0)?,
                    name: row.get::<_, String>(1)?,
                    city: row.get::<_, String>(2)?,
                    status: row.get::<_, String>(3)?,
                    dob: row.get::<_, Option<String>>(4)?,
                    time: row.get::<_, Option<String>>(5)?,
                })
            })?
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
    let search_term: String = match Input::new()
        .with_prompt("Enter part or all of the client's name")
        .interact_text()
    {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };

    let mut stmt =
        conn.prepare("SELECT id, name, city, status, dob, time FROM Clients WHERE name LIKE ?")?;
    let query_term = format!("%{}%", search_term);
    let results = stmt
        .query_map(params![query_term], |row| {
            Ok(ClientRecord {
                id: row.get::<_, i64>(0)?,
                name: row.get::<_, String>(1)?,
                city: row.get::<_, String>(2)?,
                status: row.get::<_, String>(3)?,
                dob: row.get::<_, Option<String>>(4)?,
                time: row.get::<_, Option<String>>(5)?,
            })
        })?
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
    let id: i64 = Input::new()
        .with_prompt("Enter the ID of the client (or type 0 to cancel)")
        .interact_text()
        .unwrap_or_default();

    if id == 0 {
        println!("Action cancelled.");
        return Ok(());
    }

    let fields = &["Name", "City", "Status (Active/Refused)"];
    let selection = match Select::new()
        .with_prompt("What would you like to update?")
        .items(fields)
        .default(0)
        .interact()
    {
        Ok(s) => s,
        Err(_) => return Ok(()),
    };

    match selection {
        0 => {
            if let Ok(new_name) = Input::<String>::new()
                .with_prompt("Enter new Name")
                .interact_text()
            {
                conn.execute(
                    "UPDATE Clients SET name = ? WHERE id = ?",
                    params![new_name, id],
                )?;
                println!("Client Name updated successfully.");
            }
        }
        1 => {
            if let Ok(new_city) = Input::<String>::new()
                .with_prompt("Enter new City")
                .interact_text()
            {
                conn.execute(
                    "UPDATE Clients SET city = ? WHERE id = ?",
                    params![new_city, id],
                )?;
                println!("Client City updated successfully.");
            }
        }
        2 => {
            if let Ok(new_status) = Input::<String>::new()
                .with_prompt("Enter new Status (Active/Refused)")
                .interact_text()
            {
                conn.execute(
                    "UPDATE Clients SET status = ? WHERE id = ?",
                    params![new_status, id],
                )?;
                println!("Client Status updated successfully.");
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn delete_client(conn: &Connection) -> Result<()> {
    let id: i64 = Input::new()
        .with_prompt("Enter the ID of the client to DELETE (or 0 to cancel)")
        .interact_text()
        .unwrap_or_default();

    if id == 0 {
        println!("Action cancelled.");
        return Ok(());
    }

    let confirmed = Confirm::new()
        .with_prompt("Are you sure you want to delete this client and all their readings? This cannot be undone.")
        .default(false)
        .interact()
        .unwrap_or(false);

    if confirmed {
        conn.execute("DELETE FROM Readings WHERE client_id = ?", params![id])?;
        conn.execute("DELETE FROM Clients WHERE id = ?", params![id])?;
        println!("Client and associated records deleted permanently.");
    } else {
        println!("Deletion cancelled.");
    }
    Ok(())
}

fn launch_wizard() -> Option<(String, String, String, String, String, u32)> {
    println!("\n--- Run New Astrology Reading ---");

    let name: String = Input::new()
        .with_prompt("Enter Client Name")
        .interact_text()
        .ok()?;

    let date: String = loop {
        let input: String = match Input::new()
            .with_prompt("Enter Date of Birth (DD/MM/YYYY)")
            .interact_text()
        {
            Ok(val) => val,
            Err(_) => return None,
        };
        if NaiveDate::parse_from_str(&input, "%d/%m/%Y").is_ok() {
            break input;
        }
        println!(
            "{}",
            style("Invalid date format. Please use DD/MM/YYYY (e.g., 15/08/1990).").red()
        );
    };

    let time: String = loop {
        let input: String = match Input::new()
            .with_prompt("Enter Time of Birth (e.g., 10:45 AM or 14:30)")
            .interact_text()
        {
            Ok(val) => val,
            Err(_) => return None,
        };
        if NaiveTime::parse_from_str(&input, "%I:%M %p").is_ok()
            || NaiveTime::parse_from_str(&input, "%H:%M").is_ok()
        {
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

    let city: String = Input::new()
        .with_prompt("Enter City of Birth")
        .interact_text()
        .ok()?;
    let question: String = Input::new()
        .with_prompt("Enter the Querent's Question")
        .interact_text()
        .ok()?;

    let words_str: String = Input::new()
        .with_prompt("Enter desired reading length in words (e.g., 500)")
        .interact_text()
        .unwrap_or_else(|_| "500".to_string());
    let target_words: u32 = words_str.parse().unwrap_or(500);

    Some((name, date, time, city, question, target_words))
}

pub struct ReadingParams {
    pub name: String,
    pub date: String,
    pub time: String,
    pub city: String,
    pub question: String,
    pub target_words: u32,
}

fn fast_track_reading(conn: &Connection) -> Result<Option<ReadingParams>> {
    println!("\n--- Fast-Track Existing Client ---");
    let search_name: String = match Input::new()
        .with_prompt("Enter the Name of the client (or type 'cancel' to exit)")
        .interact_text()
    {
        Ok(val) => val,
        Err(_) => return Ok(None),
    };

    if search_name.trim().eq_ignore_ascii_case("cancel") {
        println!("Action cancelled.");
        return Ok(None);
    }

    let mut stmt =
        conn.prepare("SELECT id, name, city, status, dob, time FROM Clients WHERE name LIKE ?")?;
    let query_term = format!("%{}%", search_name);
    let results: Vec<ClientRecord> = stmt
        .query_map(params![query_term], |row| {
            Ok(ClientRecord {
                id: row.get::<_, i64>(0)?,
                name: row.get::<_, String>(1)?,
                city: row.get::<_, String>(2)?,
                status: row.get::<_, String>(3)?,
                dob: row.get::<_, Option<String>>(4)?,
                time: row.get::<_, Option<String>>(5)?,
            })
        })?
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

        let selected_id_str: String = match Input::new()
            .with_prompt("Enter the specific ID of the correct match from this detailed list")
            .interact_text()
        {
            Ok(val) => val,
            Err(_) => return Ok(None),
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

        let question: String = match Input::new()
            .with_prompt("Enter the Querent's NEW Question")
            .interact_text()
        {
            Ok(val) => val,
            Err(_) => return Ok(None),
        };

        let words_str: String = Input::new()
            .with_prompt("Enter desired reading length in words (e.g., 500)")
            .interact_text()
            .unwrap_or_else(|_| "500".to_string());
        let target_words: u32 = words_str.parse().unwrap_or(500);

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

fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .connection_verbose(false)
        .tcp_keepalive(None)
        .pool_idle_timeout(std::time::Duration::from_secs(5))
        .pool_max_idle_per_host(1)
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .expect("Network Initialization Error")
}

fn parse_birth_datetime(date_str: &str, time_str: &str) -> Option<NaiveDateTime> {
    let date = NaiveDate::parse_from_str(date_str, "%d/%m/%Y").ok()?;
    let time = NaiveTime::parse_from_str(time_str, "%I:%M %p")
        .or_else(|_| NaiveTime::parse_from_str(time_str, "%H:%M"))
        .ok()?;
    Some(NaiveDateTime::new(date, time))
}

async fn save_and_open_html_report(name: &str, final_reading: &str) {
    tokio::fs::create_dir_all("readings")
        .await
        .unwrap_or_default();

    let html_content = format!(
        "<!DOCTYPE html>\n<html>\n<head>\n\
        <meta charset=\"UTF-8\">\n<title>Vedic Reading - {}</title>\n\
        <style>\n\
        body {{ font-family: system-ui, sans-serif; max-width: 800px; margin: 40px auto; line-height: 1.8; color: #333; padding: 20px; background-color: #fcfcfc; border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.1); }}\n\
        </style>\n</head>\n<body>\n\
        <h1>Vedic Reading for {}</h1>\n\
        <pre style=\"white-space: pre-wrap; font-family: inherit;\">{}</pre>\n\
        </body>\n</html>",
        name, name, final_reading
    );

    let clean_name = utils::sanitize_filename(name);
    let date_suffix = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_{}.html", clean_name, date_suffix);

    if let Ok(mut absolute_path) = std::env::current_dir() {
        absolute_path.push("readings");
        absolute_path.push(&filename);

        if let Ok(mut file) = tokio::fs::File::create(&absolute_path).await {
            let _ = file.write_all(html_content.as_bytes()).await;
            println!(
                "Reading generated! Opening in browser at: {}",
                absolute_path.display()
            );
            let _ = open::that(&absolute_path);
            return;
        }
    }

    // Fallback to terminal output if file save fails
    println!(
        "\n--- AI Vedic Reading for {} ---\n{}\n--- End of Reading ---",
        name, final_reading
    );
}

async fn execute_reading_flow(
    conn: &Connection,
    name: String,
    date_str: String,
    time_str: String,
    city: String,
    question: String,
    target_words: u32,
) {
    let client = build_http_client();

    let naive_dt = match parse_birth_datetime(&date_str, &time_str) {
        Some(dt) => dt,
        None => {
            eprintln!("Invalid Date/Time Format. Please verify date (DD/MM/YYYY) and time.");
            return;
        }
    };

    println!("\nInitializing Astrology Workflow...");

    println!("Resolving Location and Historical Timezone for {}...", city);
    let (lat, lon, offset) = match geo::get_location_data(&city, naive_dt).await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Location Resolution Error: {}", e);
            return;
        }
    };

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

    let house_selection = Select::new()
        .with_prompt("Select House System")
        .items(house_options)
        .default(1)
        .interact()
        .unwrap_or(1);

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
    let final_reading = match api::call_gemini_with_retry(
        &client,
        system_prompt.clone(),
        anonymized_user_prompt.clone(),
        "gemini-3.1-flash-lite",
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

    // Save and open HTML report
    save_and_open_html_report(&name, &final_reading).await;
}

fn wait_for_enter() {
    print!("\nPress Enter to return to Main Menu...");
    io::stdout().flush().unwrap();
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
}

#[tokio::main]
async fn main() {
    let conn = init_db().expect("Database Initialization Error");
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
            &conn,
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
            "🔍 Search Client by Name",
            "✏️ Edit a Client's Details",
            "❌ Delete a Client",
            "🚪 Exit Program",
        ];

        let selection = match Select::new()
            .with_prompt("Main Menu - Select an Action")
            .items(menu_options)
            .default(0)
            .interact()
        {
            Ok(s) => s,
            Err(_) => break,
        };

        match selection {
            0 => {
                if let Some((name, date, time, city, question, target_words)) = launch_wizard() {
                    execute_reading_flow(&conn, name, date, time, city, question, target_words)
                        .await;
                }
                wait_for_enter();
            }
            1 => {
                if let Ok(Some(params)) = fast_track_reading(&conn) {
                    execute_reading_flow(
                        &conn,
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
                let _ = search_clients(&conn);
                wait_for_enter();
            }
            4 => {
                let _ = edit_client(&conn);
                wait_for_enter();
            }
            5 => {
                let _ = delete_client(&conn);
                wait_for_enter();
            }
            6 => {
                println!("Exiting Program... Goodbye!");
                break;
            }
            _ => unreachable!(),
        }
    }
}
