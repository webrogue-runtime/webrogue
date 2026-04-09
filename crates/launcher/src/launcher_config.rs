use std::path::PathBuf;

#[async_trait::async_trait]
pub trait LauncherConfig: Send + Sync {
    fn storage_path(&self) -> PathBuf;
    async fn launch(
        &self,
        sdp_offer: String,
        on_sdp_answer: Box<dyn FnOnce(String) + Send>,
    ) -> anyhow::Result<()>;
}
