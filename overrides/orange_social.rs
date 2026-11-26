use freya::prelude::*;

use crate::components::{Button, ButtonSize, ButtonVariant, TextInput};
use crate::orange_api::{
    self, OrangeFriendRequest, OrangeSession, OrangeUser,
};
use crate::theme::{colors, rounded, text_size};

#[derive(PartialEq)]
pub(super) struct OrangeSocialPanel;

impl Component for OrangeSocialPanel {
    fn render(&self) -> impl IntoElement {
        let mut session = use_state(|| None::<OrangeSession>);
        let mut ready = use_state(|| false);
        let mut busy = use_state(|| false);
        let mut username = use_state(String::new);
        let mut password = use_state(String::new);
        let mut registering = use_state(|| false);
        let mut search = use_state(String::new);
        let mut search_results = use_state(Vec::<OrangeUser>::new);
        let mut friends = use_state(Vec::<OrangeUser>::new);
        let mut incoming = use_state(Vec::<OrangeFriendRequest>::new);
        let mut message = use_state(|| None::<String>);

        use_hook({
            let session = session;
            let ready = ready;
            let friends = friends;
            let incoming = incoming;
            let message = message;
            move || {
                spawn_global(async move {
                    match orange_api::load_saved_session().await {
                        Ok(Some(saved)) => {
                            let token = saved.access_token.clone();
                            session.set(Some(saved));
                            let (friend_result, request_result) = tokio::join!(
                                orange_api::friends(&token),
                                orange_api::incoming_requests(&token)
                            );
                            match friend_result {
                                Ok(value) => friends.set(value),
                                Err(error) if error.starts_with("AUTH:") => {
                                    orange_api::clear_saved_session().await;
                                    session.set(None);
                                    message.set(Some("Your Orange session expired. Please sign in again.".to_string()));
                                }
                                Err(error) => message.set(Some(error)),
                            }
                            if let Ok(value) = request_result {
                                incoming.set(value);
                            }
                        }
                        Ok(None) => {}
                        Err(error) => message.set(Some(error)),
                    }
                    ready.set(true);
                });
            }
        });

        let signed_in = session.read().clone();
        let friend_rows = friends.read().clone();
        let request_rows = incoming.read().clone();
        let result_rows = search_results.read().clone();
        let current_message = message.read().clone();

        let mut content = rect()
            .vertical()
            .width(Size::fill())
            .height(Size::fill())
            .overflow(Overflow::Scroll)
            .padding(Gaps::new_all(16.))
            .spacing(12.);

        content = content.child(
            rect()
                .vertical()
                .width(Size::fill())
                .spacing(3.)
                .child(
                    label()
                        .text("Orange Network")
                        .font_size(text_size(19.))
                        .font_weight(FontWeight::BOLD)
                        .color(colors::fg_primary()),
                )
                .child(
                    label()
                        .text("Your Orange account and friends")
                        .font_size(text_size(12.))
                        .color(colors::fg_secondary()),
                ),
        );

        if !*ready.read() {
            content = content.child(
                label()
                    .text("Connecting to Orange...")
                    .font_size(text_size(13.))
                    .color(colors::fg_secondary()),
            );
            return content.into_element();
        }

        if let Some(active) = signed_in {
            let token = active.access_token.clone();
            let sign_out_session = session;
            let sign_out_message = message;
            let sign_out_busy = busy;
            let sign_out = Button::new()
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Small)
                .on_press(move |_| {
                    let token = token.clone();
                    let session = sign_out_session;
                    let message = sign_out_message;
                    let busy = sign_out_busy;
                    spawn_global(async move {
                        busy.set(true);
                        let _ = orange_api::logout(&token).await;
                        orange_api::clear_saved_session().await;
                        session.set(None);
                        message.set(Some("Signed out of Orange.".to_string()));
                        busy.set(false);
                    });
                })
                .text("Sign out");

            content = content.child(
                rect()
                    .horizontal()
                    .width(Size::fill())
                    .cross_align(Alignment::Center)
                    .spacing(8.)
                    .child(
                        rect()
                            .vertical()
                            .width(Size::flex(1.))
                            .spacing(2.)
                            .child(
                                label()
                                    .text(active.user.username.clone())
                                    .font_size(text_size(15.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .color(colors::fg_primary()),
                            )
                            .child(
                                label()
                                    .text("Signed in to Orange")
                                    .font_size(text_size(11.))
                                    .color(colors::success()),
                            ),
                    )
                    .child(sign_out),
            );

            let refresh_token = active.access_token.clone();
            let refresh_friends = friends;
            let refresh_requests = incoming;
            let refresh_message = message;
            let refresh_busy = busy;
            content = content.child(
                Button::new()
                    .secondary()
                    .size(ButtonSize::Small)
                    .on_press(move |_| {
                        let token = refresh_token.clone();
                        let friends = refresh_friends;
                        let requests = refresh_requests;
                        let message = refresh_message;
                        let busy = refresh_busy;
                        spawn_global(async move {
                            busy.set(true);
                            let (friend_result, request_result) = tokio::join!(
                                orange_api::friends(&token),
                                orange_api::incoming_requests(&token)
                            );
                            match friend_result {
                                Ok(value) => friends.set(value),
                                Err(error) => message.set(Some(error)),
                            }
                            match request_result {
                                Ok(value) => requests.set(value),
                                Err(error) => message.set(Some(error)),
                            }
                            busy.set(false);
                        });
                    })
                    .text("Refresh friends"),
            );

            content = content.child(section_heading("Friend requests"));
            if request_rows.is_empty() {
                content = content.child(empty_hint("No incoming requests."));
            } else {
                let mut request_list = rect().vertical().width(Size::fill()).spacing(5.);
                for request in request_rows {
                    let request_id = request.id;
                    let request_name = request.username.clone();
                    let accept_name = request_name.clone();
                    let reject_name = request_name.clone();
                    let accept_token = active.access_token.clone();
                    let accept_friends = friends;
                    let accept_requests = incoming;
                    let accept_message = message;
                    let accept_busy = busy;
                    let reject_token = active.access_token.clone();
                    let reject_requests = incoming;
                    let reject_message = message;
                    let reject_busy = busy;
                    request_list = request_list.child(
                        rect()
                            .horizontal()
                            .width(Size::fill())
                            .padding(Gaps::new_symmetric(8., 7.))
                            .spacing(6.)
                            .cross_align(Alignment::Center)
                            .corner_radius(rounded(8.))
                            .background(colors::component_bg())
                            .child(
                                label()
                                    .text(request_name.clone())
                                    .font_size(text_size(12.))
                                    .color(colors::fg_primary())
                                    .width(Size::flex(1.)),
                            )
                            .child(
                                Button::new()
                                    .primary()
                                    .size(ButtonSize::Small)
                                    .on_press(move |_| {
                                        let token = accept_token.clone();
                                        let friends = accept_friends;
                                        let requests = accept_requests;
                                        let message = accept_message;
                                        let busy = accept_busy;
                                        spawn_global(async move {
                                            busy.set(true);
                                            match orange_api::accept_request(&token, request_id).await {
                                                Ok(()) => {
                                                    let (friend_result, request_result) = tokio::join!(
                                                        orange_api::friends(&token),
                                                        orange_api::incoming_requests(&token)
                                                    );
                                                    if let Ok(value) = friend_result { friends.set(value); }
                                                    if let Ok(value) = request_result { requests.set(value); }
                                                    message.set(Some(format!("You and {accept_name} are now friends.")));
                                                }
                                                Err(error) => message.set(Some(error)),
                                            }
                                            busy.set(false);
                                        });
                                    })
                                    .text("Accept"),
                            )
                            .child(
                                Button::new()
                                    .variant(ButtonVariant::Ghost)
                                    .size(ButtonSize::Small)
                                    .on_press(move |_| {
                                        let token = reject_token.clone();
                                        let requests = reject_requests;
                                        let message = reject_message;
                                        let busy = reject_busy;
                                        spawn_global(async move {
                                            busy.set(true);
                                            match orange_api::reject_request(&token, request_id).await {
                                                Ok(()) => {
                                                    requests.set(
                                                        orange_api::incoming_requests(&token).await.unwrap_or_default()
                                                    );
                                                    message.set(Some(format!("Request from {reject_name} declined.")));
                                                }
                                                Err(error) => message.set(Some(error)),
                                            }
                                            busy.set(false);
                                        });
                                    })
                                    .text("Decline"),
                            ),
                    );
                }
                content = content.child(request_list);
            }

            content = content.child(section_heading("Friends"));
            if friend_rows.is_empty() {
                content = content.child(empty_hint("Your Orange friends will appear here."));
            } else {
                let mut friend_list = rect().vertical().width(Size::fill()).spacing(5.);
                for friend in friend_rows {
                    let friend_id = friend.id;
                    let friend_name = friend.username.clone();
                    let remove_token = active.access_token.clone();
                    let remove_friends = friends;
                    let remove_message = message;
                    let remove_busy = busy;
                    friend_list = friend_list.child(
                        rect()
                            .horizontal()
                            .width(Size::fill())
                            .padding(Gaps::new_symmetric(8., 7.))
                            .spacing(6.)
                            .cross_align(Alignment::Center)
                            .corner_radius(rounded(8.))
                            .background(colors::component_bg())
                            .child(
                                label()
                                    .text(friend_name.clone())
                                    .font_size(text_size(12.))
                                    .color(colors::fg_primary())
                                    .width(Size::flex(1.)),
                            )
                            .child(
                                Button::new()
                                    .variant(ButtonVariant::Ghost)
                                    .size(ButtonSize::Small)
                                    .on_press(move |_| {
                                        let token = remove_token.clone();
                                        let friends = remove_friends;
                                        let message = remove_message;
                                        let busy = remove_busy;
                                        spawn_global(async move {
                                            busy.set(true);
                                            match orange_api::remove_friend(&token, friend_id).await {
                                                Ok(()) => {
                                                    friends.set(orange_api::friends(&token).await.unwrap_or_default());
                                                    message.set(Some(format!("{friend_name} removed from your friends.")));
                                                }
                                                Err(error) => message.set(Some(error)),
                                            }
                                            busy.set(false);
                                        });
                                    })
                                    .text("Remove"),
                            ),
                    );
                }
                content = content.child(friend_list);
            }

            content = content.child(section_heading("Find Orange players"));
            content = content.child(
                TextInput::new(search)
                    .width(Size::fill())
                    .height(Size::px(36.))
                    .font_size(text_size(12.))
                    .placeholder("Search username (2–16 characters)"),
            );

            let search_token = active.access_token.clone();
            let search_value = search;
            let search_results_state = search_results;
            let search_message = message;
            let search_busy = busy;
            content = content.child(
                Button::new()
                    .secondary()
                    .size(ButtonSize::Small)
                    .on_press(move |_| {
                        let token = search_token.clone();
                        let query = search_value.read().trim().to_string();
                        let results = search_results_state;
                        let message = search_message;
                        let busy = search_busy;
                        spawn_global(async move {
                            if query.len() < 2 {
                                message.set(Some("Enter at least 2 characters to search.".to_string()));
                                return;
                            }
                            busy.set(true);
                            match orange_api::search_users(&token, &query).await {
                                Ok(value) => {
                                    results.set(value);
                                    message.set(Some("Orange player search complete.".to_string()));
                                }
                                Err(error) => message.set(Some(error)),
                            }
                            busy.set(false);
                        });
                    })
                    .text("Search"),
            );

            if !result_rows.is_empty() {
                let mut results = rect().vertical().width(Size::fill()).spacing(5.);
                for result in result_rows {
                    let result_name = result.username.clone();
                    let add_token = active.access_token.clone();
                    let add_results = search_results;
                    let add_message = message;
                    let add_busy = busy;
                    results = results.child(
                        rect()
                            .horizontal()
                            .width(Size::fill())
                            .padding(Gaps::new_symmetric(8., 7.))
                            .spacing(6.)
                            .cross_align(Alignment::Center)
                            .corner_radius(rounded(8.))
                            .background(colors::component_bg())
                            .child(
                                label()
                                    .text(result_name.clone())
                                    .font_size(text_size(12.))
                                    .color(colors::fg_primary())
                                    .width(Size::flex(1.)),
                            )
                            .child(
                                Button::new()
                                    .primary()
                                    .size(ButtonSize::Small)
                                    .on_press(move |_| {
                                        let token = add_token.clone();
                                        let username = result_name.clone();
                                        let results = add_results;
                                        let message = add_message;
                                        let busy = add_busy;
                                        spawn_global(async move {
                                            busy.set(true);
                                            match orange_api::send_friend_request(&token, &username).await {
                                                Ok(target) => {
                                                    message.set(Some(format!("Friend request sent to {target}.")));
                                                    results.set(Vec::new());
                                                }
                                                Err(error) => message.set(Some(error)),
                                            }
                                            busy.set(false);
                                        });
                                    })
                                    .text("Add"),
                            ),
                    );
                }
                content = content.child(results);
            }
        } else {
            content = content.child(
                rect()
                    .horizontal()
                    .width(Size::fill())
                    .spacing(6.)
                    .child(
                        Button::new()
                            .variant(if !*registering.read() { ButtonVariant::Primary } else { ButtonVariant::Ghost })
                            .size(ButtonSize::Small)
                            .on_press(move |_| registering.set(false))
                            .text("Sign in"),
                    )
                    .child(
                        Button::new()
                            .variant(if *registering.read() { ButtonVariant::Primary } else { ButtonVariant::Ghost })
                            .size(ButtonSize::Small)
                            .on_press(move |_| registering.set(true))
                            .text("Create account"),
                    ),
            );

            content = content
                .child(
                    TextInput::new(username)
                        .width(Size::fill())
                        .height(Size::px(38.))
                        .font_size(text_size(13.))
                        .placeholder("Orange username"),
                )
                .child(
                    TextInput::new(password)
                        .mode(InputMode::new_password())
                        .width(Size::fill())
                        .height(Size::px(38.))
                        .font_size(text_size(13.))
                        .placeholder("Password (10–128 characters)"),
                );

            let auth_username = username;
            let auth_password = password;
            let auth_registering = registering;
            let auth_session = session;
            let auth_ready = ready;
            let auth_friends = friends;
            let auth_incoming = incoming;
            let auth_message = message;
            let auth_busy = busy;
            let button_label = if *registering.read() { "Create Orange account" } else { "Sign in to Orange" };
            content = content.child(
                Button::new()
                    .primary()
                    .size(ButtonSize::Large)
                    .width(Size::fill())
                    .disabled(*busy.read())
                    .on_press(move |_| {
                        let name = auth_username.read().trim().to_string();
                        let pass = auth_password.read().clone();
                        let is_register = *auth_registering.read();
                        let session = auth_session;
                        let ready = auth_ready;
                        let friends = auth_friends;
                        let incoming = auth_incoming;
                        let message = auth_message;
                        let busy = auth_busy;
                        spawn_global(async move {
                            if name.len() < 3 || name.len() > 16 {
                                message.set(Some("Username must be 3–16 characters.".to_string()));
                                return;
                            }
                            if pass.len() < 10 || pass.len() > 128 {
                                message.set(Some("Password must be 10–128 characters.".to_string()));
                                return;
                            }
                            busy.set(true);
                            let result = if is_register {
                                orange_api::register(&name, &pass).await
                            } else {
                                orange_api::login(&name, &pass).await
                            };
                            match result {
                                Ok(auth) => {
                                    let token = auth.access_token.clone();
                                    let save_error = orange_api::save_session(&auth).await.err();
                                    session.set(Some(auth));
                                    let (friend_result, request_result) = tokio::join!(
                                        orange_api::friends(&token),
                                        orange_api::incoming_requests(&token)
                                    );
                                    if let Ok(value) = friend_result { friends.set(value); }
                                    if let Ok(value) = request_result { incoming.set(value); }
                                    message.set(save_error.or_else(|| Some("Welcome to Orange Client!".to_string())));
                                    ready.set(true);
                                }
                                Err(error) => message.set(Some(error)),
                            }
                            busy.set(false);
                        });
                    })
                    .text(button_label),
            );

            content = content.child(
                label()
                    .text("Orange accounts are separate from your Microsoft Minecraft account.")
                    .font_size(text_size(11.))
                    .color(colors::fg_secondary()),
            );
        }

        if let Some(current_message) = current_message {
            content = content.child(
                rect()
                    .width(Size::fill())
                    .padding(Gaps::new_symmetric(8., 10.))
                    .corner_radius(rounded(8.))
                    .background(colors::component_bg())
                    .child(
                        label()
                            .text(current_message)
                            .font_size(text_size(11.))
                            .color(colors::fg_secondary()),
                    ),
            );
        }

        if *busy.read() {
            content = content.child(
                label()
                    .text("Contacting Orange...")
                    .font_size(text_size(11.))
                    .color(colors::fg_secondary()),
            );
        }

        content.into_element()
    }
}

fn section_heading(title: &'static str) -> Element {
    label()
        .text(title)
        .font_size(text_size(13.))
        .font_weight(FontWeight::SEMIBOLD)
        .color(colors::fg_primary())
        .into_element()
}

fn empty_hint(message: &'static str) -> Element {
    rect()
        .width(Size::fill())
        .padding(Gaps::new_symmetric(8., 10.))
        .corner_radius(rounded(8.))
        .background(colors::component_bg())
        .child(
            label()
                .text(message)
                .font_size(text_size(11.))
                .color(colors::fg_secondary()),
        )
        .into_element()
}
