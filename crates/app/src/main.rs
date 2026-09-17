use gpui_kit::{
    AppContext, SharedString, TitlebarOptions, WindowOptions,
    component::{Root, Theme, ThemeRegistry},
};
use shared::db::DbPool;
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::path::PathBuf;

use crate::app::AppShell;
pub mod app;

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("Failed to get database url: {0}")]
    GetDatabaseUrlEnv(String),

    #[error("Failed to start database: {0}")]
    StartDatabaseError(String),
}

async fn get_db() -> Result<Pool<Sqlite>, AppError> {
    dotenv::dotenv().ok();
    let url =
        dotenv::var("DATABASE_URL").map_err(|e| AppError::GetDatabaseUrlEnv(e.to_string()))?;

    let pool = SqlitePoolOptions::new()
        .connect(&url)
        .await
        .map_err(|e| AppError::StartDatabaseError(e.to_string()))?;

    Ok(pool)
}

#[tokio::main]
async fn main() {
    let app = gpui_kit::application().with_assets(gpui_kit::assets::Assets);

    let pool = get_db().await.expect("Failed to get database");

    app.run(move |cx| {
        gpui_kit::init(cx);
        cx.set_global(DbPool(pool.clone()));

        let dark_name = SharedString::from("Alduin");

        if let Err(err) = ThemeRegistry::watch_dir(PathBuf::from("./themes"), cx, move |cx| {
            let registry = ThemeRegistry::global(cx);

            if let Some(theme) = registry.themes().get(&dark_name).cloned() {
                Theme::global_mut(cx).font_family = "Geist Mono".into();

                Theme::global_mut(cx).apply_config(&theme);
            }
        }) {
            eprintln!("Failed to watch themes directory: {}", err);
        }

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some(SharedString::new("Targetz")),
                        ..Default::default()
                    }),
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
