use async_trait::async_trait;
use tauri::Runtime;
use tauri_plugin_background_service::{BackgroundService, ServiceContext, ServiceError};

/// The IRC engine lives in the main process; this service only holds the
/// Android foreground notification open so the OS keeps the process alive.
pub struct KeepAlive;

#[async_trait]
impl<R: Runtime> BackgroundService<R> for KeepAlive {
    async fn init(&mut self, _ctx: &ServiceContext<R>) -> Result<(), ServiceError> {
        Ok(())
    }

    async fn run(&mut self, ctx: &ServiceContext<R>) -> Result<(), ServiceError> {
        ctx.shutdown.cancelled().await;
        Ok(())
    }
}
