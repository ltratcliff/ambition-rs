use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use axum::{routing::{get, post}, http::StatusCode, Router, Json, extract::Path};
use axum::response::Html;
use rusqlite::{params, Connection};
use minijinja::{Environment, context};
use tokio_cron_scheduler::{JobScheduler, Job};

#[derive(Debug, Clone, Copy)]
enum Ambition {
    Unmotivated = 0,
    Motivated = 1,
}

impl Ambition {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Ambition::Unmotivated),
            1 => Some(Ambition::Motivated),
            _ => None,
        }
    }

    fn to_u8(self) -> u8 {
        self as u8
    }
}

impl Display for Ambition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Ambition::Motivated => write!(f, "Motivated"),
            Ambition::Unmotivated => write!(f, "Unmotivated"),
        }
    }
}

fn init_db() {
    // Path to the database file
    let db_path = "ambitions.db";

    // Check if the database file exists
    if !fs::metadata(db_path).is_ok() {
        println!("Database does not exist, initializing...");

        // Open a connection
        let conn = Connection::open(db_path).expect("Failed to create database");

        // Create the table(s)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ambitions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mood INTEGER NOT NULL,
                timestamp TEXT NOT NULL
            );",
            [],
        )
            .expect("Failed to create ambitions table");

        tracing::info!("Database initialized successfully.");
    } else {
        tracing::info!("Database already exists.");
    }
}

fn check_previous_day() {
    let conn = Connection::open("ambitions.db").unwrap();

    // Get yesterday's date in YYYY-MM-DD format
    let now = chrono::Utc::now();
    let yesterday = now.date_naive().pred_opt().unwrap(); // Get previous day
    let yesterday_str = yesterday.format("%Y-%m-%d").to_string();

    // Check if an entry exists for yesterday
    let mut stmt = conn.prepare("SELECT id FROM ambitions WHERE timestamp = ?1").unwrap();
    let entry_exists = stmt.exists(params![yesterday_str]).unwrap();

    if !entry_exists {
        // Insert new entry with mood=0 (Unmotivated)
        conn.execute(
            "INSERT INTO ambitions (mood, timestamp) VALUES (?1, ?2)",
            params![0, yesterday_str],
        ).unwrap();

        tracing::info!("Added missing entry for yesterday ({}) with mood=Unmotivated", yesterday_str);
    } else {
        tracing::info!("Entry for yesterday ({}) already exists", yesterday_str);
    }
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    init_db();
    check_previous_day();

    // Set up the scheduler to run check_previous_day daily at midnight
    let scheduler = JobScheduler::new().await.unwrap();

    // Add a job that runs at midnight every day
    let job = Job::new_async("0 5 0 * * *", move |_uuid, _l| {
        Box::pin(async move {
            tracing::info!("Running scheduled check_previous_day at midnight");
            check_previous_day();
        })
    });

    // Add the job to the scheduler
    scheduler.add(job.expect("REASON")).await.unwrap();

    // Start the scheduler in the background
    tokio::spawn(async move {
        scheduler.start().await.unwrap();
    });
    tracing::info!("Scheduler started, will run check_previous_day at midnight daily");

    // build our application with a route
    let app = Router::new()
        // Create a nested router with the "ambition" prefix
        .nest("/ambition", Router::new()
            .route("/", get(root))
            .route("/health", get(get_health))
            .route("/api/{value}", post(post_ambition)));


    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();

}

async fn root() -> (StatusCode, Html<String>) {
    tracing::info!("GET /");

    // Set up minijinja environment
    let mut env = Environment::new();

    // Load template from file
    let template_content = fs::read_to_string("templates/index.html")
        .expect("Failed to read template file");

    // Add template to environment
    env.add_template("index", &template_content)
        .expect("Failed to add template");

    // Get the current mood from the database (reusing logic from get_ambition)
    let conn = Connection::open("ambitions.db").unwrap();
    let mut stmt = conn.prepare("SELECT mood FROM ambitions ORDER BY id DESC LIMIT 1").unwrap();

    let mood_result = stmt.query_map([], |row| {
        let mood_value: u8 = row.get(0)?;
        Ok(mood_value)
    });

    // Default to Unmotivated if no records or error
    let mood = match mood_result {
        Ok(mut rows) => {
            if let Some(Ok(mood_value)) = rows.next() {
                Ambition::from_u8(mood_value).unwrap_or(Ambition::Unmotivated)
            } else {
                Ambition::Unmotivated
            }
        },
        Err(_) => Ambition::Unmotivated,
    };

    // Check if an entry exists for today
    let now = chrono::Utc::now();
    let today = now.format("%Y-%m-%d").to_string();
    let mut stmt = conn.prepare("SELECT id FROM ambitions WHERE timestamp = ?1").unwrap();
    let entry_exists = stmt.exists(params![today]).unwrap();

    // Render the template with the current mood and entry_exists flag
    let rendered = env.get_template("index")
        .expect("Failed to get template")
        .render(context!(
            current_mood => mood.to_string(),
            entry_exists => entry_exists
        ))
        .expect("Failed to render template");

    (StatusCode::OK, Html(rendered))
}

async fn get_health() -> (StatusCode, &'static str) {
    tracing::info!("GET /health");
    (StatusCode::OK, "OK")
}


// POST handler for /ambition/api/:value
async fn post_ambition(Path(value): Path<u8>) -> (StatusCode, Json<HashMap<String, bool>>) {
    tracing::info!("POST /ambition/api/{}", value);

    match Ambition::from_u8(value) {
        Some(mood) => {
            let mut map = HashMap::new();
            map.insert(mood.to_string(), true);
            write_to_db(mood).await;
            (StatusCode::OK, Json(map))
        },
        None => {
            let mut map = HashMap::new();
            map.insert("error".to_string(), false);
            (StatusCode::BAD_REQUEST, Json(map))
        }
    }
}

async fn write_to_db(mood: Ambition) {
    let conn = Connection::open("ambitions.db").unwrap();

    // Get today's date in YYYY-MM-DD format
    let now = chrono::Utc::now();
    let today = now.format("%Y-%m-%d").to_string();

    // Check if an entry exists for today
    let mut stmt = conn.prepare("SELECT id FROM ambitions WHERE timestamp = ?1").unwrap();
    let entry_exists = stmt.exists(params![today]).unwrap();

    if entry_exists {
        // Update existing entry
        conn.execute(
            "UPDATE ambitions SET mood = ?1 WHERE timestamp = ?2",
            params![mood.to_u8(), today],
        ).unwrap();
    } else {
        // Insert new entry
        conn.execute(
            "INSERT INTO ambitions (mood, timestamp) VALUES (?1, ?2)",
            params![mood.to_u8(), today],
        ).unwrap();
    }
}
