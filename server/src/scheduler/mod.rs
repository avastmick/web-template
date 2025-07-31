use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::info;

use crate::services::OAuthService;

/// Set up the OAuth state cleanup scheduler
///
/// # Errors
///
/// Returns an error if the scheduler fails to initialize or start.
pub async fn setup_oauth_cleanup_scheduler(
    oauth_service: &Arc<OAuthService>,
) -> Result<(), Box<dyn std::error::Error>> {
    let scheduler = JobScheduler::new().await.map_err(|e| {
        tracing::error!("Failed to create job scheduler: {:?}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    let oauth_service_for_cleanup = oauth_service.clone();
    scheduler
        .add(
            Job::new_async("0 */10 * * * *", move |_uuid, _l| {
                let oauth_service = oauth_service_for_cleanup.clone();
                Box::pin(async move {
                    match oauth_service.cleanup_expired_states().await {
                        Ok(deleted_count) => {
                            if deleted_count > 0 {
                                tracing::info!("Cleaned up {} expired OAuth states", deleted_count);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to cleanup expired OAuth states: {:?}", e);
                        }
                    }
                })
            })
            .map_err(|e| {
                tracing::error!("Failed to create cleanup job: {:?}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to add cleanup job to scheduler: {:?}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    scheduler.start().await.map_err(|e| {
        tracing::error!("Failed to start job scheduler: {:?}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    info!("OAuth state cleanup job scheduled to run every 10 minutes");
    Ok(())
}
