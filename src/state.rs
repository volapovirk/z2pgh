use crate::email_client::EmailClient;
use async_std::sync::Arc;

use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct State {
    pub db_pool: Arc<PgPool>,
    pub email_client: Arc<EmailClient>,
}

impl State {
    pub fn new(db_pool: PgPool, email_client: EmailClient) -> Self {
        Self {
            db_pool: Arc::new(db_pool),
            email_client: Arc::new(email_client),
        }
    }
}
