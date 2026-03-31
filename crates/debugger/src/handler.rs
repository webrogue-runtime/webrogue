#[async_trait::async_trait]
pub trait EventHandler {
    async fn handle(&self);
}
