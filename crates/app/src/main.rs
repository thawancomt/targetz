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
use std::{path::PathBuf, str::FromStr, time::Duration};

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

    let options = SqliteConnectOptions::from_str(&url)
        .map_err(|e| AppError::StartDatabaseError(e.to_string()))?
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        // era max_connections(1) — vale subir pra permitir leitores
        // concorrentes junto do único writer que o WAL mode já suporta
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(|e| AppError::StartDatabaseError(e.to_string()))?;

    Ok(pool)
}

fn main() {
    let app = gpui_kit::application().with_assets(gpui_kit::assets::Assets);

    // ÚNICA mudança real: get_db().await vira block_on(get_db()).
    // Isso roda ANTES do GPUI assumir a thread principal — nesse
    // instante ainda não existe event loop nenhum disputando nada,
    // então driblar a future até o fim aqui é seguro. Não sobra
    // runtime nenhum vivo depois desta linha: o backend `runtime-smol`
    // do sqlx não precisa de um runtime "guardado" — o reactor dele já
    // roda numa thread global própria, independente de quem faz .await.
    let pool = block_on(get_db()).expect("Failed to get database");

    app.run(move |cx| {
        gpui_kit::init(cx);
        cx.set_global(DbPool(pool.clone()));

        let dark_name = SharedString::from("Dark");

        if let Err(err) = ThemeRegistry::watch_dir(PathBuf::from("./themes"), cx, move |cx| {
            let registry = ThemeRegistry::global(cx);

            if let Some(theme) = registry.themes().get(&dark_name).cloned() {
                Theme::global_mut(cx).font_family = "Geist Mono".into();
                Theme::global_mut(cx).apply_config(&theme);
            }
        }) {
            eprintln!("Failed to watch themes directory: {}", err);
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
