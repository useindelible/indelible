use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use ind_domain::{DomainEventId, UserId};
use serde::Deserialize;
use sqlx::PgPool;
use tokio::sync::broadcast;

const CHANNEL_CAPACITY: usize = 256;
const MAX_SUBSCRIBERS_PER_USER: usize = 10;
pub const PG_NOTIFY_CHANNEL: &str = "indelible_domain_events";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomainEventNotification {
    pub user_id: UserId,
    pub event_id: DomainEventId,
}

#[derive(Clone, Default)]
pub struct RealtimeHub {
    inner: Arc<Mutex<HashMap<UserId, UserChannel>>>,
}

struct UserChannel {
    sender: broadcast::Sender<DomainEventNotification>,
    subscribers: usize,
}

pub struct RealtimeSubscription {
    hub: RealtimeHub,
    user_id: UserId,
    receiver: broadcast::Receiver<DomainEventNotification>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealtimeSubscribeError {
    TooManySubscribers,
}

impl RealtimeHub {
    pub fn new() -> Self {
        Self::default()
    }

    #[expect(
        clippy::expect_used,
        reason = "registry/state mutex never held across await; poisoning implies an already-fatal prior panic"
    )]
    pub fn subscribe(
        &self,
        user_id: UserId,
    ) -> Result<RealtimeSubscription, RealtimeSubscribeError> {
        let mut channels = self.inner.lock().expect("realtime hub mutex poisoned");
        let channel = channels.entry(user_id).or_insert_with(|| {
            let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
            UserChannel {
                sender,
                subscribers: 0,
            }
        });
        if channel.subscribers >= MAX_SUBSCRIBERS_PER_USER {
            return Err(RealtimeSubscribeError::TooManySubscribers);
        }
        channel.subscribers += 1;
        Ok(RealtimeSubscription {
            hub: self.clone(),
            user_id,
            receiver: channel.sender.subscribe(),
        })
    }

    #[expect(
        clippy::expect_used,
        reason = "registry/state mutex never held across await; poisoning implies an already-fatal prior panic"
    )]
    pub fn publish(&self, notification: DomainEventNotification) {
        let Some(sender) = self
            .inner
            .lock()
            .expect("realtime hub mutex poisoned")
            .get(&notification.user_id)
            .map(|channel| channel.sender.clone())
        else {
            return;
        };
        let _ = sender.send(notification);
    }

    #[expect(
        clippy::expect_used,
        reason = "registry/state mutex never held across await; poisoning implies an already-fatal prior panic"
    )]
    fn unsubscribe(&self, user_id: UserId) {
        let mut channels = self.inner.lock().expect("realtime hub mutex poisoned");
        let Some(channel) = channels.get_mut(&user_id) else {
            return;
        };
        channel.subscribers = channel.subscribers.saturating_sub(1);
        if channel.subscribers == 0 {
            channels.remove(&user_id);
        }
    }
}

impl RealtimeSubscription {
    pub async fn recv(&mut self) -> Result<DomainEventNotification, broadcast::error::RecvError> {
        self.receiver.recv().await
    }
}

impl Drop for RealtimeSubscription {
    fn drop(&mut self) {
        self.hub.unsubscribe(self.user_id);
    }
}

#[derive(Debug, Deserialize)]
struct NotifyPayload {
    user_id: uuid::Uuid,
    event_id: uuid::Uuid,
}

pub fn spawn_pg_listener(pool: PgPool, hub: RealtimeHub) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut backoff = std::time::Duration::from_millis(500);
        loop {
            match run_listener_once(pool.clone(), hub.clone()).await {
                Ok(()) => backoff = std::time::Duration::from_millis(500),
                Err(err) => {
                    tracing::warn!(error = %err, "domain event listener disconnected; reconnecting");
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(std::time::Duration::from_secs(30));
                }
            }
        }
    })
}

async fn run_listener_once(pool: PgPool, hub: RealtimeHub) -> Result<(), sqlx::Error> {
    let mut listener = sqlx::postgres::PgListener::connect_with(&pool).await?;
    listener.listen(PG_NOTIFY_CHANNEL).await?;

    loop {
        let notification = listener.recv().await?;
        match serde_json::from_str::<NotifyPayload>(notification.payload()) {
            Ok(payload) => hub.publish(DomainEventNotification {
                user_id: UserId::from_uuid(payload.user_id),
                event_id: DomainEventId::from_uuid(payload.event_id),
            }),
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    payload = notification.payload(),
                    "invalid domain event notification payload"
                );
            }
        }
    }
}
