use std::collections::HashMap;

use gloo_console::log;
use leptos::*;
use protocol::{ClientCommand, ServerCommand};
use uuid::Uuid;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{MediaStream, MediaStreamTrack, MessageEvent, WebSocket};

use crate::components::BandMember;

#[component]
pub fn Session(
    stream: ReadSignal<Option<MediaStream>>,
    set_tracks: WriteSignal<HashMap<Uuid, Vec<MediaStreamTrack>>>,
) -> impl IntoView {
    let (members, set_members) = create_signal(HashMap::new());
    let (connected, set_connected) = create_signal(false);

    let ws = WebSocket::new("ws://127.0.0.1:3000/ws").unwrap();

    let onopen = move |_: MessageEvent| set_connected.update(|c| *c = true);
    let cb = Closure::wrap(Box::new(onopen) as Box<dyn FnMut(_)>);
    ws.set_onopen(Some(cb.as_ref().unchecked_ref()));
    cb.forget();

    let onclose = move |_: MessageEvent| set_connected.update(|c| *c = false);
    let cb = Closure::wrap(Box::new(onclose) as Box<dyn FnMut(_)>);
    ws.set_onclose(Some(cb.as_ref().unchecked_ref()));
    cb.forget();

    let onmessage = move |event: MessageEvent| {
        let message = event.data().as_string().unwrap();
        match serde_json::from_str::<protocol::ServerCommand>(&message) {
            Ok(command) => match command.clone() {
                ServerCommand::AddMember(uuid, polite) => {
                    log!("===================== Add", &uuid.to_string());
                    let action = create_rw_signal(ServerCommand::AddMember(uuid, polite));
                    set_members.update(move |ms| {
                        ms.insert(uuid, action);
                    });
                }
                ServerCommand::DropMember(uuid) => {
                    log!("===================== Drop", &uuid.to_string());
                    set_tracks.update(|tracks| {
                        tracks.remove(&uuid);
                    });
                    set_members.update(move |ms| {
                        ms.remove(&uuid);
                    });
                }

                _ => {
                    log!("===================== Somethings up");
                    set_members.update(move |ms| {
                        ms.entry(command.get_uuid()).and_modify(
                            |m: &mut RwSignal<ServerCommand>| {
                                m.set(command);
                            },
                        );
                    });
                }
            },
            _ => {
                log!("NOO");
            }
        };
    };

    let cb = Closure::wrap(Box::new(onmessage) as Box<dyn FnMut(_)>);
    ws.set_onmessage(Some(cb.as_ref().unchecked_ref()));
    cb.forget();

    let (ws, _) = create_signal(ws);
    let send_message = move || {
        let ws = ws.get();
        move |message: ClientCommand| {
            let message = serde_json::to_string(&message).unwrap();
            ws.send_with_str(&message)
                .expect("Couldn't send message to signal server");
        }
    };

    view! {
    {move || if connected.get() {
        view!{<div class="has-background-success">.</div>}
    }  else {
        view!{<div class="has-background-danger">.</div>}
        }
    }
    <For
        each=move || members.get()
        key= |(k,_)| k.clone()
        children= move |(k, v)| {
            let (action, set_action) = v.split();
            view!{
            <BandMember
                uuid={k.to_owned()}
                action=action
                stream=stream
                send_message=send_message()
                add_track=move |track: MediaStreamTrack| set_tracks.update(|tracks| tracks.entry(k).or_insert(vec![]).push(track))
                update_action= move |command| set_action.set(command)
            />}
        }
    />}
}
