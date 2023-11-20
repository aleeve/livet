use super::Error;
use anyhow::Result;
use gloo::console::log;
use gloo_utils::format::JsValueSerdeExt;
use leptos::{Callable, Callback};
use protocol::ClientCommand;
use serde_json::json;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys::{AudioTrack, MediaStreamTrack, RtcConfiguration, RtcPeerConnection, TrackEvent};

#[derive(Clone)]
pub struct Rtc {
    pub connection: RtcPeerConnection,
    pub msgs: Vec<String>,
    pub send_message: Callback<ClientCommand>,
    pub uuid: Uuid,
    pub polite: bool,
    pub pending_offer: bool,
}

impl PartialEq for Rtc {
    fn eq(&self, other: &Rtc) -> bool {
        self.uuid == other.uuid && self.msgs == other.msgs && self.connection == other.connection
    }
}

impl Rtc {
    pub fn new(uuid: Uuid, send_message: Callback<ClientCommand>) -> Result<Self, Error> {
        // Configure rtc connection
        let conf = json!([
            {"urls": "stun:stun.stunprotocol.org:3478"},
            {"urls": "stun:stun.l.google.com:19302"},
        ]);

        let conf = JsValue::from_serde(&conf).map_err(|_| Error::ConfigurationError)?;
        let mut rtc_configuration = RtcConfiguration::new();
        rtc_configuration.ice_servers(&conf);

        // Setup peerconnection
        let connection = RtcPeerConnection::new_with_configuration(&rtc_configuration)
            .map_err(|_| Error::ConnectionError)?;

        Ok(Self {
            connection,
            msgs: vec![],
            polite: false,
            pending_offer: false,
            send_message,
            uuid,
        })
    }

    pub fn add_track_callback(
        &mut self,
        add_track: Callback<MediaStreamTrack>,
    ) -> Result<(), JsValue> {
        let ontrack = move |ev: TrackEvent| {
            if let Some(track) = ev.track() {
                let track = MediaStreamTrack::from(JsValue::from(track));
                add_track.call(track);
            }
        };
        let cb = Closure::wrap(Box::new(ontrack) as Box<dyn FnMut(_)>);
        self.connection
            .set_ontrack(Some(cb.as_ref().unchecked_ref()));
        cb.forget();
        Ok(())
    }
}
