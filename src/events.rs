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

// Driver Reconnect failed Events
pub struct DriverReconnectFailedNotifier {
    pub manager: Arc<Songbird>,
    pub data: Arc<serenity::prelude::RwLock<TypeMap>>,
    pub handler: Arc<serenity::prelude::Mutex<Call>>,
    pub guild_id: u64,
    pub chan_id: ChannelId,
}

// When songbird driver failed to reconnect,
// try to reconnect manually
#[async_trait]
impl VoiceEventHandler for DriverReconnectFailedNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let (handle_lock, success) = self.manager.join(self.guild_id, self.chan_id).await;

        if let Ok(_channel) = success {
            let mut handle = handle_lock.lock().await;

            handle.add_global_event(
                Event::Track(TrackEvent::Play),
                TrackStartNotifier {
                    data: self.data.clone(),
                    key: self.guild_id,
                    // chan_id,
                    // http: send_http.clone(),
                },
            );

            handle.add_global_event(
                Event::Track(TrackEvent::End),
                TrackEndNotifier {
                    handler: handle_lock.clone(),
                    data: self.data.clone(),
                    key: self.guild_id,
                    // chan_id,
                    // http: send_http.clone(),
                },
            );

            handle.add_global_event(
                Event::Core(CoreEvent::DriverReconnectFailed),
                DriverReconnectFailedNotifier {
                    manager: self.manager.clone(),
                    data: self.data.clone(),
                    handler: handle_lock.clone(),
                    guild_id: self.guild_id,
                    chan_id: self.chan_id.clone(),
                },
            );
        }

        None
    }
}
