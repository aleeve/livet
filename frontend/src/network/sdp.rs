use super::Rtc;

use anyhow::Result;
use futures::{future::join_all, Future};
use gloo::console::log;
use js_sys::Reflect;
use leptos::{spawn_local, Callable, Callback};
use protocol::{ClientCommand, ServerCommand};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Event, RtcSdpType, RtcSessionDescriptionInit, RtcSignalingState};

impl Rtc {
    pub async fn create_offer(&mut self) -> Result<String, JsValue> {
        self.msgs.push("offer".to_string());

        let offer = JsFuture::from(self.connection.create_offer())
            .await
            .unwrap();
        let offer_sdp = Reflect::get(&offer, &JsValue::from_str("sdp"))
            .unwrap()
            .as_string()
            .unwrap();

        let mut offer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
        offer_obj.sdp(&offer_sdp);

        let sld_promise = self.connection.set_local_description(&offer_obj);
        JsFuture::from(sld_promise).await.unwrap();
        self.pending_offer = true;

        Ok(offer_sdp)
    }

    pub async fn create_answer(&mut self, offer_sdp: String) -> Option<String> {
        // First we handle the glare scenario
        let mut local_future: Option<JsFuture> = None;
        if self.connection.signaling_state() != RtcSignalingState::Stable {
            if self.polite {
                //"We are polite and have pending request we rollback and let the other peer go first"
                let description = RtcSessionDescriptionInit::new(RtcSdpType::Rollback);
                let local_promise = self.connection.set_local_description(&description);
                local_future = Some(JsFuture::from(local_promise));
            } else {
                log!("The other party needs to accept my offer instead");
                return None;
            }
        };

        let mut description = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
        description.sdp(&offer_sdp);
        let sld_promise = self.connection.set_remote_description(&description);
        let sld_future = JsFuture::from(sld_promise);

        if let Some(local_future) = local_future {
            join_all(vec![sld_future, local_future]).await;
        } else {
            sld_future.await;
        }


        // And then we create an answer and move forward
        let answer = JsFuture::from(self.connection.create_answer())
            .await
            .unwrap();
        let answer_sdp = Reflect::get(&answer, &JsValue::from_str("sdp"))
            .unwrap()
            .as_string()
            .unwrap();

        let mut answer = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
        answer.sdp(&answer_sdp);

        let sld_promise = self.connection.set_local_description(&answer);
        JsFuture::from(sld_promise).await.unwrap();

        Some(answer_sdp)
    }

    pub async fn get_answer(&mut self, answer_sdp: String) -> Result<(), JsValue> {
        self.msgs.push("get answer".to_string());

        let mut answer = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
        answer.sdp(&answer_sdp);
        let srd_promise = self.connection.set_remote_description(&answer);
        JsFuture::from(srd_promise).await?;

        Ok(())
    }

    pub fn add_negotiation_callback(
        &mut self,
        renew_offer: Callback<ServerCommand>,
    ) -> Result<(), JsValue> {
        let uuid = self.uuid.clone();
        let negotioation_needed = move |_: Event| {
            renew_offer.call(ServerCommand::CreateOffer(uuid));
        };

        let cb = Closure::wrap(Box::new(negotioation_needed) as Box<dyn FnMut(_)>);
        self.connection
            .set_onnegotiationneeded(Some(cb.as_ref().unchecked_ref()));
        cb.forget();
        Ok(())
    }
}
