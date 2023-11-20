use crate::network::Rtc;
use gloo_console::log;
use leptos::*;
use protocol::{ClientCommand, ServerCommand};
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::{MediaStream, MediaStreamTrack, RtcRtpSender, RtcSignalingState};

#[component]
pub fn BandMember(
    uuid: Uuid,
    action: ReadSignal<ServerCommand>,
    #[prop(into)] send_message: Callback<ClientCommand>,
    #[prop(into)] add_track: Callback<MediaStreamTrack>,
    #[prop(into)] update_action: Callback<ServerCommand>,
    stream: ReadSignal<Option<MediaStream>>,
) -> impl IntoView {
    // Create a RTC connection object
    let mut connection = Rtc::new(uuid, send_message.clone()).unwrap();
    connection.add_track_callback(add_track).unwrap();
    connection.add_ice_callback().unwrap();
    let (connection, set_connection) = create_signal(connection);

    let update_action = move |command: ServerCommand| {
        let state = connection.get().connection.signaling_state();
        if state == RtcSignalingState::Stable {
            update_action.call(command);
        }
    };

    // TODO: The on negotiation callback flow is currently broken. Needs to be adjusted
    // so that the app follows https://blog.mozilla.org/webrtc/perfect-negotiation-in-webrtc/
    connection.get_untracked().add_negotiation_callback(update_action.into()).unwrap();


    // Make sure new local tracks, i.e device inputs, are sent over to
    // this bandmember
    create_effect(move |_| {
        let connection = connection.get_untracked();
        match stream.get() {
            Some(stream) => {
                // Get current acutal senders
                let current_tracks = connection
                    .connection
                    .get_senders()
                    .iter()
                    .map(|sender| RtcRtpSender::from(sender))
                    .filter(|s| s.track().is_some())
                    .collect::<Vec<RtcRtpSender>>();

                let wanted_tracks: Vec<MediaStreamTrack> = stream
                    .get_tracks()
                    .iter()
                    .map(|track| MediaStreamTrack::from(track))
                    .collect();

                // Remove unwanted tracks
                let remove_tracks: Vec<&RtcRtpSender> = current_tracks
                    .iter()
                    .filter(|t| !wanted_tracks.contains(&t.track().expect("Missing track")))
                    .collect();
                for track in &remove_tracks {
                    log!("Removing specific track");
                    connection.connection.remove_track(track);
                }

                // Add wanted tracks that isn't already added
                let keep_tracks: Vec<MediaStreamTrack> = current_tracks
                    .iter()
                    .filter(|t| !remove_tracks.contains(t))
                    .map(|t| MediaStreamTrack::from(t.track().expect("Missing track")))
                    .collect();
                for track in &wanted_tracks {
                    if !keep_tracks.contains(track) {
                        log!("Adding specific track");
                        connection.connection.add_track_0(track, &stream);
                    }
                }
            }
            None => log!("No stream yet"),
        };
    });

    create_resource(
        move || action.get(),
        move |action: ServerCommand| {
            let mut connection = connection.get_untracked();

            async move {
                log!(format!("{:?}", action));
                match action {
                    ServerCommand::CreateOffer(uuid) => {
                        let offer = connection
                            .create_offer()
                            .await
                            .expect("Failed to create offer");
                        send_message.call(ClientCommand::Offer(uuid, offer));
                        set_connection.set(connection);
                    }
                    ServerCommand::CreateAnswer(uuid, offer) => {
                        let should_answer = connection.polite || !connection.pending_offer;
                        if should_answer {
                            let answer = connection
                                .create_answer(offer)
                                .await
                                .expect("Failed to create anwser");
                            send_message.call(ClientCommand::Answer(uuid, answer));
                            set_connection.set(connection);
                        }
                    }
                    ServerCommand::GetAnswer(_, answer) => {
                        connection
                            .get_answer(answer)
                            .await
                            .expect("Failed to get answer");
                        set_connection.set(connection);
                    }

                    ServerCommand::AddIceCandidate(_, ice) => {
                        connection.get_ice(ice).await.expect("Failed to get ice");
                        set_connection.set(connection);
                    }

                    ServerCommand::AddMember(_, polite) => {
                        connection.polite = polite;
                        set_connection.set(connection);
                    }
                    _ => log!("Got unexpected command!"),
                }
            }
        },
    );

    view!(<div> {uuid.to_string()} </div>
        <For
            each=move || connection.get().msgs
            key=|msg| msg.clone()
            children= move |msg| view!{<p>{msg}</p>}
        />
    )
}
