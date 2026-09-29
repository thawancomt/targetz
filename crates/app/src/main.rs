use gpui_kit::{
    AppContext, Bounds, SharedString, TitlebarOptions, WindowBounds, WindowOptions, block_on,
    component::{Root, Theme, ThemeRegistry},
    point, px, size,
};
use shared::db::DbPool;
use sqlx::{
    Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{str::FromStr, time::Duration};

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
    let url = std::env::var("DATABASE_URL")
        .or_else(|_| dotenv::var("DATABASE_URL"))
        .unwrap_or_else(|_| {
            if std::path::Path::new("todos.db").exists() {
                "sqlite://todos.db".to_string()
            } else if let Ok(home) = std::env::var("HOME") {
                let dir = std::path::PathBuf::from(home).join(".local/share/targetz");
                let _ = std::fs::create_dir_all(&dir);
                format!("sqlite://{}", dir.join("todos.db").display())
            } else {
                "sqlite://todos.db".to_string()
            }
        });

    let options = SqliteConnectOptions::from_str(&url)
        .map_err(|e| AppError::StartDatabaseError(e.to_string()))?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        // era max_connections(1) — vale subir pra permitir leitores
        // concorrentes junto do único writer que o WAL mode já suporta
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(|e| AppError::StartDatabaseError(e.to_string()))?;

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .map_err(|e| AppError::StartDatabaseError(e.to_string()))?;

    Ok(pool)
}

fn main() {
    let app = gpui_kit::application().with_assets(gpui_kit::assets::Assets);

    let pool = block_on(get_db()).expect("Failed to get database");

    app.run(move |cx| {
        gpui_kit::init(cx);
        cx.set_global(DbPool(pool.clone()));

        settings::init_themes(cx);

        let dark_name = SharedString::from("Dark");
        let registry = ThemeRegistry::global(cx);
        if let Some(theme) = registry.themes().get(&dark_name).cloned() {
            Theme::global_mut(cx).font_family = "Geist Mono".into();
            Theme::global_mut(cx).radius = px(0.);
            Theme::global_mut(cx).apply_config(&theme);
            Theme::sync_base(cx);
        }

        let bound = Bounds::centered(None, size(px(1280.), px(720.)), cx);
        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some(SharedString::new("Targetz")),
                        ..Default::default()
                    }),
                    window_bounds: Some(WindowBounds::Windowed(bound)),
                    ..Default::default()
                },
                |window, cx| {
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
