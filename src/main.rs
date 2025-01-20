use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use axum::{routing::get, http::StatusCode, Router, Json};
use axum::response::Html;
use rusqlite::{params, Connection};

enum Ambition {
    Motivated,
    Unmotivated
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
                mood TEXT NOT NULL,
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

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    init_db();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/health", get(get_health))
        .route("/api", get(get_mood))
        .route("/sm", get(set_motivated))
        .route("/sun", get(set_unmotivated));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

async fn root() -> (StatusCode, Html<String>) {
    tracing::info!("GET /");
    let rendered = "Hello, World!";
    (StatusCode::OK, Html(rendered.to_string()))
}

async fn get_mood() -> Json<HashMap<String,bool>> {
    tracing::info!("GET /api");
    let mood = Ambition::Unmotivated;
    match mood {
        Ambition::Motivated => {
            tracing::info!("We are Motivated");
            let mut map = HashMap::new();
            map.insert(mood.to_string(), true);
            Json(map)
        },
        Ambition::Unmotivated => {
            tracing::info!("We are Unmotivated");
            let mut map = HashMap::new();
            map.insert(mood.to_string(), true);
            Json(map)
        }
    }
}

async fn set_motivated() -> Json<HashMap<String,bool>> {
    tracing::info!("GET /set_motivated");
    let mood = Ambition::Motivated;
    let mut map = HashMap::new();
    map.insert(mood.to_string(), true);
    write_to_db(mood.to_string().as_str()).await;
    Json(map)
}

async fn set_unmotivated() -> Json<HashMap<String,bool>> {
    tracing::info!("GET /set_unmotivated");
    let mood = Ambition::Unmotivated;
    let mut map = HashMap::new();
    map.insert(mood.to_string(), true);
    write_to_db(mood.to_string().as_str()).await;
    Json(map)
}

async fn get_health() -> (StatusCode, &'static str) {
    tracing::info!("GET /health");
    (StatusCode::OK, "OK")
}

async fn write_to_db(mood: &str) {
    let conn = Connection::open("ambitions.db").unwrap();
    conn.execute(
        "INSERT INTO ambitions (mood, timestamp) VALUES (?1, ?2)",
        params![mood, chrono::Utc::now().timestamp() ],
    )
        .unwrap();
}

