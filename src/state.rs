use async_std::sync::Arc;

use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct State {
    pub db_pool: Arc<PgPool>,
}

impl State {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool: Arc::new(db_pool),
        }
    }
}
