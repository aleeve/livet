use std::collections::HashMap;

use crate::audio::{get_devices, AudioGraph, InputDeviceInfo};
use crate::components::LevelMeter;
use gloo_console::log;
use leptos::*;
use uuid::Uuid;
use web_sys::{self, MediaStream, MediaStreamTrack};

#[component]
pub fn MixerBoard(
    #[prop(into)] set_stream: Callback<MediaStream>,
    tracks: ReadSignal<HashMap<Uuid, Vec<MediaStreamTrack>>>,
) -> impl IntoView {
    // This is for the device selector
    let (input_device, set_input_device) = create_signal("default".to_string());

    // Setup audio graph
    let mut graph = AudioGraph::new().expect("Failed to create audio ctx");
    graph.suspend().unwrap();
    let (graph, set_graph) = create_signal(graph);

    create_effect(move |_| {
        let tracks = tracks.get();
        let mut graph = graph.get_untracked();
        for (owner, track) in tracks {
            for t in track {
                graph.add_input(owner, t).unwrap();
            }
        }
        set_graph.set_untracked(graph);
    });

    create_resource(
        move || input_device.get(),
        move |device| {
            let mut graph = graph.clone().get_untracked();
            let set_graph = set_graph.clone();
            async move {
                log!("Switching dev");
                graph.add_device(&device).await.unwrap();
                graph.connect().unwrap();
                set_graph.set(graph);
            }
        },
    );

    let on_change = move |e| {
        if event_target_checked(&e) {
            set_graph.update(|graph| {
                graph.resume().unwrap();
            })
        } else {
            set_graph.update(|graph| {
                graph.suspend().unwrap();
            })
        }
    };

    // Update outgoing stream when audio graph stream changes
    let out_stream = create_memo(move |_| graph.get().stream().unwrap());
    create_effect(move |_| {
        let stream = out_stream.get();
        set_stream.call(stream);
    });

    let r_graph = graph.clone();
    let remote_tracks = move || r_graph.get().remote_tracks;

    view! {
    <section class="hero is-primary">
        <div class="hero-body">
            <SelectDevices set_input_device=set_input_device/>
            <div class="field">
                <input
                    id="isLiveSwitch"
                    class="switch is-danger is-rounded"
                    type="checkbox"
                    on:change=on_change />
                <label for="isLiveSwitch"> live </label>
            </div>
            <div>
                {"This is where I configure my inputs and sound"}
            </div>
        </div>
        <For
         each=move || remote_tracks()
         key=|track| track.label.clone()
         children= move |track| {
            log!("Remote is real");
            view!{
                <div>{track.label}</div>
                <LevelMeter analyser= track.analyser.clone()  />
            }
        }
        />
        <LevelMeter analyser=graph.get().analyser />
    </section>
    }
}

#[component]
fn SelectDevices(set_input_device: WriteSignal<String>) -> impl IntoView {
    let devices = create_resource(
        || (),
        |_| async move {
            let devices = get_devices().await.expect("Failed");
            devices
                .into_iter()
                .filter(|dev| dev.kind == "audioinput")
                .collect::<Vec<InputDeviceInfo>>()
        },
    );

    view! {
    <Transition
     fallback = || view!{<p> Loading... </p>}
    >
        {
            move || {
                if ! devices.loading().get() {
                    view!{
                        <div class="field">
                            <label class="label">Sound input</label>
                            <div  class="select" >
                                <select on:change=move |e| set_input_device.set(event_target_value(&e))>
                                    <For
                                     each = move || devices.get().unwrap()
                                     key=|dev: &InputDeviceInfo| dev.deviceId.clone()
                                     children=move |dev| {
                                        view!{<option value={dev.deviceId.clone()}>{&dev.label}</option>}
                                    }/>
                                </select>
                            </div>
                        </div>
                    }
                } else {
                    view!{<div> WAT </div>}
                }
            }
        }
        </Transition>
    }
}
