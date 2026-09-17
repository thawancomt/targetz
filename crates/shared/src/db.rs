use sqlx::{Pool, Sqlite};

pub struct DbPool(pub Pool<Sqlite>);
impl gpui_kit::Global for DbPool {}
