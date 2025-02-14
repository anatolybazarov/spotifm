#[macro_use]
extern crate actix_web;
use core::time;
use std::sync::{Arc, Mutex};
use std::{thread, sync::mpsc::{Receiver, SyncSender, sync_channel}};
use librespot::core::authentication::Credentials;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;
use librespot::core::spotify_id::{self, SpotifyId};
use librespot::playback::audio_backend;
use librespot::playback::config::{AudioFormat, PlayerConfig};
use librespot::playback::mixer::NoOpVolume;
use librespot::playback::player::{Player, PlayerEvent};
use librespot_oauth::{OAuthToken, get_access_token};
use rustls;
mod db;
mod rest;
mod signals;
mod config;
mod announce;
use config::SpotifmConfig;
const BACKEND: &str = "pulseaudio";
#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider().install_default().expect("Failed to install rustls crypto provider");
    let mut tracks_played = 0;
    let args: Vec<String> = std::env::args().collect();
    let db = db::SpotifyDatabase::new();
    let config = Arc::new(Mutex::new(SpotifmConfig::load(args.get(1).unwrap().clone())));
    let (session, token) = create_session(&config).await;
    let session = Arc::new(Mutex::new(session));
    let token = Arc::new(Mutex::new(token));
    let (rest_tx, rest_rx): (SyncSender<PlayerEvent>, Receiver<PlayerEvent>) = sync_channel(100);
    let (signal_tx, signal_rx): (SyncSender<signals::SignalMessage>, Receiver<signals::SignalMessage>) = sync_channel(100);
    signals::start(signal_tx.clone());
    rest::start(rest_tx.clone(), config.clone(), session.clone(), db.clone(), token.clone());
    db::populate(config.lock().unwrap().uris.clone(), session.clone(), db.clone());
    while db.len() == 0 {
        thread::sleep(time::Duration::from_millis(10));
    }
    'track_list: loop {
        db.advance_track();
        match db.current_track() {
            Err(err) => panic!("{}", err.unwrap()),
            Ok(track) => {
                tracks_played += 1;
                let uri = format!("spotify:track:{}", track.id);
                let spotify_id = SpotifyId::from_uri(uri.as_str()).unwrap();
                eprintln!("Playing: {} - {}", track.track, track.artists.join(", "));
                eprintln!("Spotify ID: {:?}", spotify_id);
                let backend = audio_backend::find(None).unwrap();
                let player = Player::new(
                    PlayerConfig::default(),
                    session.lock().unwrap().clone(),
                    Box::new(NoOpVolume),
                    move || {
                        backend(None, AudioFormat::default())
                    }
                );

                announce::announcements(config.clone(), &track, tracks_played);
                player.load(spotify_id, true, 0);
                player.play();

                let mut player_rx = player.get_player_event_channel();
                match db.next_track() {
                    Err(err) => eprintln!("Preload error: {}", err),
                    Ok(track) => player.preload(track.spotify_id()),
                }
                loop {
                    thread::sleep(time::Duration::from_millis(100));
                    if let Ok(signal_event) = signal_rx.try_recv() {
                        if let signals::SignalMessage::SessionExpired = signal_event {
                            eprintln!("Session expired, creating new session...");
                            let (new_session, new_token) = create_session(&config).await;
                            *session.lock().unwrap() = new_session;
                            *token.lock().unwrap() = new_token;
                            continue 'track_list;
                        }
                    }
                    let mut events: Vec<PlayerEvent> = Vec::new();
                    if let Ok(e) = rest_rx.try_recv() {
                        events.push(e);
                    }
                    if let Ok(e) = player_rx.try_recv() {
                        events.push(e);
                    }
                    for event in events {
                        match event {
                            PlayerEvent::EndOfTrack { .. } => {
                                continue 'track_list;
                            }
                            PlayerEvent::Stopped { .. } => {
                                player.stop();
                                continue 'track_list;
                            }
                            PlayerEvent::PlayRequestIdChanged { play_request_id, .. } => {
                                let spotify_id = SpotifyId {
                                    id: play_request_id as u128,
                                    item_type: librespot::core::spotify_id::SpotifyItemType::Track,
                                };
                                player.preload(spotify_id);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
pub async fn create_session(config: &Arc<Mutex<SpotifmConfig>>) -> (Session, OAuthToken) {
    let _config = config.lock().unwrap();
    let session_config = SessionConfig::default();
    const SPOTIFY_CLIENT_ID: &str = "65b708073fc0480ea92a077233ca87bd";
    const SPOTIFY_REDIRECT_URI: &str = "http://127.0.0.1:8898/login";
    let scopes = vec!["streaming"];
    let token = get_access_token(SPOTIFY_CLIENT_ID, SPOTIFY_REDIRECT_URI, scopes).expect("failed to get access token");
    let access_token = token.access_token;
    let credentials = Credentials::with_access_token(&access_token);
    let session = Session::new(session_config, None);
    Session::connect(&session, credentials, false).await
        .map_err(|err| { eprintln!("Error creating session: {}", err.to_string()) })
        .unwrap();
    let token = OAuthToken {
        access_token: access_token,
        ..token
    };
    (session, token)
}
