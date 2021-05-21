use crate::define::*;
use crate::import::*;

// Track Start Event
pub struct TrackStartNotifier {
    pub data: Arc<serenity::prelude::RwLock<TypeMap>>,
    pub key: u64,
    // chan_id: ChannelId,
    // http: Arc<Http>,
}

// When Track pause and volume change until track pausing
// Track's volume will not change when Track will resume
// So, We defined Track Start Event then re-Set the volume
#[async_trait]
impl VoiceEventHandler for TrackStartNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(&[(_state, track)]) = ctx {
            let data = self.data.read().await;
            if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
                let mut sympho_global = sympho_global_mutex.write().await;
                let sympho_data = sympho_global.entry(self.key).or_insert(SymphoData {
                    volume: 1.0,
                    ..Default::default()
                });
                let _ = track.set_volume(sympho_data.volume);
            }
        }

        None
    }
}

// Track End Event
pub struct TrackEndNotifier {
    pub handler: Arc<serenity::prelude::Mutex<Call>>,
    pub data: Arc<serenity::prelude::RwLock<TypeMap>>,
    pub key: u64,
    // chan_id: ChannelId,
    // http: Arc<Http>,
}

// We have to go next track when Track playing ended
// Sympho will check the ownself queue when raised Track End Event
#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let mut handler = self.handler.lock().await;
        let data = self.data.read().await;

        if let Some(sympho_global_mutex) = data.get::<SymphoGlobal>() {
            let mut sympho_global = sympho_global_mutex.write().await;
            let sympho_data = sympho_global.entry(self.key).or_insert(SymphoData {
                volume: 1.0,
                ..Default::default()
            });

            if sympho_data.queue.len() != 0 {
                let track_sympho = sympho_data.queue[0].clone();
                let url = track_sympho.url.clone();

                sympho_data.queue.remove(0);

                let source = if let Ok(source) = get_source(url).await {
                    source
                } else {
                    return None;
                };

                sympho_data.queue_duration -= track_sympho.duration;

                sympho_data.current = Some((
                    play_from_source(&mut handler, source, sympho_data.volume),
                    track_sympho,
                ));
            } else {
                sympho_data.current = None;
                return None;
            }
        }

        None
    }
}
