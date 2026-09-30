#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use directories::ProjectDirs;
use gpui_kit::{
    AppContext, Bounds, SharedString, TitlebarOptions, WindowBounds, WindowOptions, block_on,
    component::Root,
    px, size,
};
use shared::db::DbPool;
use sqlx::{
    Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::time::Duration;

use crate::app::AppShell;
pub mod app;
pub mod home;

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("Failed to get database url: {0}")]
    GetDatabaseUrlEnv(String),

    #[error("Failed to start database: {0}")]
    StartDatabaseError(String),
}

async fn get_db() -> Result<Pool<Sqlite>, AppError> {
    dotenv::dotenv().ok();

    let custom_url = std::env::var("DATABASE_URL").ok().or_else(|| dotenv::var("DATABASE_URL").ok());

    let pool = if let Some(url) = custom_url {
        use std::str::FromStr;
        let options = SqliteConnectOptions::from_str(&url)
            .map_err(|e| AppError::StartDatabaseError(e.to_string()))?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5));

        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(|e| AppError::StartDatabaseError(e.to_string()))?
    } else {
        let db_path = if std::path::Path::new("todos.db").exists() {
            std::path::PathBuf::from("todos.db")
        } else if let Some(dirs) = ProjectDirs::from("software", "whatever", "targetz") {
            let data_dir = dirs.data_dir();
            let _ = std::fs::create_dir_all(data_dir);
            data_dir.join("todos.db")
        } else {
            std::path::PathBuf::from("todos.db")
        };

        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5));

        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(|e| AppError::StartDatabaseError(e.to_string()))?
    };

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .map_err(|e| AppError::StartDatabaseError(e.to_string()))?;

    Ok(pool)
}

#[tokio::main]
async fn main() {
    let app = gpui_kit::application().with_assets(gpui_kit::assets::Assets);

    let pool = block_on(get_db()).expect("Failed to get database");

    app.run(move |cx| {
        gpui_kit::init(cx);
        cx.set_global(DbPool(pool.clone()));

        settings::init_themes(cx);
        settings::SettingsManager::apply(cx);

        let bound = Bounds::centered(None, size(px(1280.), px(720.)), cx);
        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some(SharedString::new("Targetz")),
                        ..Default::default()
                    }),
                    window_bounds: Some(WindowBounds::Windowed(bound)),
                    app_id: Some("software.whatever.targetz".to_string()),
                    ..Default::default()
                },
                |window, cx| {
                    window.set_window_title("Targetz");
                    cx.set_global(DbPool(pool));
                    let view = AppShell::view(window, cx);
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )
            .expect("failed")
        })
        .detach();
    });
}
