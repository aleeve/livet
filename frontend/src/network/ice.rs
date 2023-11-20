use gloo::console::log;
use js_sys::JSON;
use leptos::{Callable, Callback};
use protocol::ClientCommand;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RtcIceCandidate, RtcIceCandidateInit, RtcPeerConnection, RtcPeerConnectionIceEvent};

use super::Rtc;

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdpMid: String,
    pub sdpMLineIndex: u16,
}

impl Rtc {
    pub fn add_ice_callback(&mut self) -> Result<(), JsValue> {
        let send_message = self.send_message.clone();
        let uuid = self.uuid.clone();

        let onicecandidate = move |ev: RtcPeerConnectionIceEvent| {
            match ev.candidate() {
                Some(candidate) => {
                    let json_obj_candidate = candidate.to_json();
                    let res = JSON::stringify(&json_obj_candidate).unwrap_throw();
                    let js_ob = String::from(res.clone());
                    send_message.call(ClientCommand::IceCandidate(
                        uuid.clone(),
                        serde_json::to_string(
                            &serde_json::from_str::<IceCandidate>(&js_ob).unwrap(),
                        )
                        .unwrap(),
                    ));
                }
                None => {
                    log!("No candidate yet");
                }
            };
        };

        let cb = Closure::wrap(Box::new(onicecandidate) as Box<dyn FnMut(_)>);
        self.connection
            .set_onicecandidate(Some(cb.as_ref().unchecked_ref()));
        cb.forget();
        Ok(())
    }

    pub async fn get_ice(&mut self, ice: String) -> Result<(), JsValue> {
        log!(self.connection.signaling_state());
        let icecandidate = serde_json::from_str::<IceCandidate>(&ice).map_err(|_| {
            let message = format!("Could not deserialize Ice Candidate ");
            JsValue::from_str(&message)
        })?;

        let mut rtc_ice_init = RtcIceCandidateInit::new("");
        rtc_ice_init.candidate(&icecandidate.candidate);
        rtc_ice_init.sdp_m_line_index(Some(icecandidate.sdpMLineIndex));
        rtc_ice_init.sdp_mid(Some(&icecandidate.sdpMid));

        match RtcIceCandidate::new(&rtc_ice_init) {
            Ok(x) => {
                JsFuture::from(
                    self.connection
                        .add_ice_candidate_with_opt_rtc_ice_candidate(Some(&x)),
                )
                .await?;
            }
            Err(e) => {
                log!("Ice Candidate Addition error, {} | {:?}", ice, &e);
                return Err(e);
            }
        };
        Ok(())
    }
}
