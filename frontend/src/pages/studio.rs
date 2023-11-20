use std::{collections::HashMap, time::Duration};

use crate::components::{MixerBoard, Session};
use gloo_console::log;
use leptos::*;
use web_sys::{MediaStream, MediaStreamTrack};

#[component]
pub fn Studio() -> impl IntoView {
    let (stream, set_stream) = create_signal(None);
    let (tracks, set_tracks) = create_signal(HashMap::new());

    view! {
        <MixerBoard
            set_stream= move |stream: MediaStream| set_stream.set(Some(stream))
            tracks=tracks
        />
        <div class="section">
            // <div class="columns">
            //     <div class="column"><Fader/></div>
            //     <div class="column"><Fader/></div>
            //     <div class="column"><Fader/></div>
            //     <div class="column"><Fader/></div>
            // </div>
        </div>
        <Session stream=stream set_tracks=set_tracks/>
    }
}
