use std::path::PathBuf;

use freya::prelude::*;

use crate::components::{Button, ButtonSize, ButtonVariant, LocalImage, TextInput};
use crate::hooks::{try_default_account, use_current_account};
use crate::orange_api::{
    self, OrangeFriendRequest, OrangeSession, OrangeUser,
};
use crate::theme::{colors, rounded, text_size};

#[derive(Clone, PartialEq)]
struct CapeGalleryEntry {
    username: String,
    updated_at: String,
    preview: PathBuf,
}

#[derive(PartialEq)]
pub(super) struct OrangeSocialPanel;

impl Component for OrangeSocialPanel {
    fn render(&self) -> impl IntoElement {
        let mut session = use_state(|| None::<OrangeSession>);
        let mut busy = use_state(|| false);
        let mut search = use_state(String::new);
        let mut search_results = use_state(Vec::<OrangeUser>::new);
        let mut friends = use_state(Vec::<OrangeUser>::new);
        let mut incoming = use_state(Vec::<OrangeFriendRequest>::new);
        let mut message = use_state(|| None::<String>);
        let mut cape_gallery = use_state(Vec::<CapeGalleryEntry>::new);
        let mut selected_cape_file = use_state(|| None::<PathBuf>);
        let mut cape_message = use_state(|| Some("Static 64×32 PNG · free · one cape per Minecraft account".to_string()));
        let mut cape_busy = use_state(|| false);

        let current_account_query = use_current_account();
        let minecraft_account = try_default_account(&current_account_query);
        let saved_session = session.read().clone();
        let signed_in = saved_session.filter(|saved| {
            minecraft_account
                .as_ref()
                .is_some_and(|account| saved.minecraft_account_id == Some(account.id))
        });
        let cape_session = signed_in.clone();
        let friend_rows = friends.read().clone();
        let request_rows = incoming.read().clone();
        let result_rows = search_results.read().clone();
        let current_message = message.read().clone();

        let mut content = rect()
            .vertical()
            .width(Size::fill())
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
                        .text("Your Minecraft account and friends")
                        .font_size(text_size(12.))
                        .color(colors::fg_secondary()),
                ),
        );

        if let Some(active) = signed_in {
            let token = active.access_token.clone();
            let mut sign_out_session = session;
            let mut sign_out_message = message;
            let mut sign_out_busy = busy;
            let sign_out = Button::new()
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Small)
                .on_press(move |_| {
                    let token = token.clone();
                    let mut session = sign_out_session;
                    let mut message = sign_out_message;
                    let mut busy = sign_out_busy;
                    spawn_global(async move {
                        busy.set(true);
                        let _ = orange_api::logout(&token).await;
                        orange_api::clear_saved_session().await;
                        session.set(None);
                        message.set(Some("Disconnected from Orange Friends.".to_string()));
                        busy.set(false);
                    });
                })
                .text("Disconnect");

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
                                    .font_weight(FontWeight::SEMI_BOLD)
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
            let mut refresh_friends = friends;
            let mut refresh_requests = incoming;
            let mut refresh_message = message;
            let mut refresh_busy = busy;
            content = content.child(
                Button::new()
                    .secondary()
                    .size(ButtonSize::Small)
                    .on_press(move |_| {
                        let token = refresh_token.clone();
                        let mut friends = refresh_friends;
                        let mut requests = refresh_requests;
                        let mut message = refresh_message;
                        let mut busy = refresh_busy;
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
                    let mut accept_friends = friends;
                    let mut accept_requests = incoming;
                    let mut accept_message = message;
                    let mut accept_busy = busy;
                    let reject_token = active.access_token.clone();
                    let mut reject_requests = incoming;
                    let mut reject_message = message;
                    let mut reject_busy = busy;
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
                                        let mut friends = accept_friends;
                                        let mut requests = accept_requests;
                                        let mut message = accept_message;
                                        let mut busy = accept_busy;
                                        let name = accept_name.clone();
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
                                                    message.set(Some(format!("You and {name} are now friends.")));
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
                                        let mut requests = reject_requests;
                                        let mut message = reject_message;
                                        let mut busy = reject_busy;
                                        let name = reject_name.clone();
                                        spawn_global(async move {
                                            busy.set(true);
                                            match orange_api::reject_request(&token, request_id).await {
                                                Ok(()) => {
                                                    requests.set(
                                                        orange_api::incoming_requests(&token).await.unwrap_or_default()
                                                    );
                                                    message.set(Some(format!("Request from {name} declined.")));
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
                    let mut remove_friends = friends;
                    let mut remove_message = message;
                    let mut remove_busy = busy;
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
                                        let mut friends = remove_friends;
                                        let mut message = remove_message;
                                        let mut busy = remove_busy;
                                        let name = friend_name.clone();
                                        spawn_global(async move {
                                            busy.set(true);
                                            match orange_api::remove_friend(&token, friend_id).await {
                                                Ok(()) => {
                                                    friends.set(orange_api::friends(&token).await.unwrap_or_default());
                                                    message.set(Some(format!("{name} removed from your friends.")));
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
            let mut search_value = search;
            let mut search_results_state = search_results;
            let mut search_message = message;
            let mut search_busy = busy;
            content = content.child(
                Button::new()
                    .secondary()
                    .size(ButtonSize::Small)
                    .on_press(move |_| {
                        let token = search_token.clone();
                        let query = search_value.read().trim().to_string();
                        let mut results = search_results_state;
                        let mut message = search_message;
                        let mut busy = search_busy;
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
                    let mut add_results = search_results;
                    let mut add_message = message;
                    let mut add_busy = busy;
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
                                        let mut results = add_results;
                                        let mut message = add_message;
                                        let mut busy = add_busy;
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
        } else if let Some(account) = minecraft_account {
            if account.is_microsoft() && !account.needs_sign_in() && !account.access_token.is_empty() {
                content = content.child(
                    label()
                        .text(format!("Signed into Minecraft as {}", account.username))
                        .font_size(text_size(13.))
                        .color(colors::success()),
                );
                content = content.child(
                    label()
                        .text("Use your existing Microsoft sign-in. No separate Orange account or password is needed.")
                        .font_size(text_size(11.))
                        .color(colors::fg_secondary()),
                );

                let account_id = account.id;
                let mut connect_session = session;
                let mut connect_friends = friends;
                let mut connect_requests = incoming;
                let mut connect_message = message;
                let mut connect_busy = busy;
                content = content.child(
                    Button::new()
                        .primary()
                        .size(ButtonSize::Large)
                        .width(Size::fill())
                        .disabled(*busy.read())
                        .on_press(move |_| {
                            let mut session = connect_session;
                            let mut friends = connect_friends;
                            let mut requests = connect_requests;
                            let mut message = connect_message;
                            let mut busy = connect_busy;
                            spawn_global(async move {
                                busy.set(true);
                                let refreshed = match crate::launcher::state() {
                                    Ok(launcher) => launcher
                                        .auth
                                        .account_for_launch(account_id)
                                        .await
                                        .map_err(|error| error.to_string()),
                                    Err(error) => Err(error.to_string()),
                                };

                                match refreshed {
                                    Ok(minecraft) if minecraft.is_microsoft() && !minecraft.access_token.is_empty() => {
                                        match orange_api::minecraft_login(&minecraft.access_token, account_id).await {
                                            Ok(auth) => {
                                                let token = auth.access_token.clone();
                                                let save_error = orange_api::save_session(&auth).await.err();
                                                session.set(Some(auth));
                                                let (friend_result, request_result) = tokio::join!(
                                                    orange_api::friends(&token),
                                                    orange_api::incoming_requests(&token)
                                                );
                                                if let Ok(value) = friend_result { friends.set(value); }
                                                if let Ok(value) = request_result { requests.set(value); }
                                                message.set(save_error.or_else(|| Some(format!("Connected as {}.", minecraft.username))));
                                            }
                                            Err(error) => {
                                                let message_text = error.strip_prefix("AUTH:").unwrap_or(&error).trim().to_string();
                                                message.set(Some(message_text));
                                            }
                                        }
                                    }
                                    Ok(_) => message.set(Some("Your Microsoft sign-in has expired. Sign in again in Accounts.".to_string())),
                                    Err(error) => message.set(Some(error)),
                                }
                                busy.set(false);
                            });
                        })
                        .text("Connect Orange Friends"),
                );
            } else {
                content = content.child(
                    label()
                        .text("Sign in with a Microsoft Minecraft account to add friends. Offline accounts cannot use Orange Friends.")
                        .font_size(text_size(12.))
                        .color(colors::fg_secondary()),
                );
            }
        } else {
            content = content.child(
                label()
                    .text("Sign in with Microsoft in the launcher's Accounts section to use Orange Friends. You do not need to create an Orange account.")
                    .font_size(text_size(12.))
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

        // Orange Client's own cape gallery and publisher; no OneClient marketplace.
        content = content.child(section_heading("Free Orange Capes"));
        content = content.child(
            label()
                .text("Browse community capes or publish your own static 64×32 PNG. Uploading replaces your current cape.")
                .font_size(text_size(11.))
                .color(colors::fg_secondary()),
        );

        let mut refresh_gallery = cape_gallery;
        let mut refresh_status = cape_message;
        let mut refresh_busy = cape_busy;
        let own_uuid = cape_session.as_ref().and_then(|saved| saved.minecraft_account_id);
        content = content.child(
            Button::new()
                .secondary()
                .size(ButtonSize::Small)
                .disabled(*cape_busy.read())
                .on_press(move |_| {
                    let mut gallery = refresh_gallery;
                    let mut status = refresh_status;
                    let mut busy = refresh_busy;
                    spawn_global(async move {
                        busy.set(true);
                        match orange_api::community_capes().await {
                            Ok(capes) => {
                                let cache = std::env::temp_dir().join("OrangeClient").join("community-capes");
                                let mut entries = Vec::new();
                                if tokio::fs::create_dir_all(&cache).await.is_ok() {
                                    for cape in capes.into_iter()
                                        .filter(|cape| Some(cape.minecraft_uuid) != own_uuid)
                                        .take(8)
                                    {
                                        match orange_api::download_cape_texture(&cape.texture_url).await {
                                            Ok(bytes) => {
                                                let path = cache.join(format!("{}.png", cape.minecraft_uuid.simple()));
                                                if tokio::fs::write(&path, bytes).await.is_ok() {
                                                    entries.push(CapeGalleryEntry {
                                                        username: cape.username,
                                                        updated_at: cape.updated_at,
                                                        preview: path,
                                                    });
                                                }
                                            }
                                            Err(error) => {
                                                status.set(Some(error));
                                            }
                                        }
                                    }
                                }
                                gallery.set(entries);
                                status.set(Some("Community cape list updated.".to_string()));
                            }
                            Err(error) => status.set(Some(error)),
                        }
                        busy.set(false);
                    });
                })
                .text("Refresh community capes"),
        );

        let gallery_rows = cape_gallery.read().clone();
        if gallery_rows.is_empty() {
            content = content.child(empty_hint("No cape previews loaded yet. Select Refresh community capes."));
        } else {
            for cape in gallery_rows {
                content = content.child(
                    rect()
                        .horizontal()
                        .width(Size::fill())
                        .padding(Gaps::new_symmetric(8., 7.))
                        .spacing(10.)
                        .cross_align(Alignment::Center)
                        .corner_radius(rounded(8.))
                        .background(colors::component_bg())
                        .child(
                            rect()
                                .width(Size::px(68.))
                                .height(Size::px(52.))
                                .corner_radius(rounded(6.))
                                .overflow(Overflow::Clip)
                                .child(LocalImage::new(cape.preview.clone(), 128, false)),
                        )
                        .child(
                            rect()
                                .vertical()
                                .width(Size::flex(1.))
                                .spacing(3.)
                                .child(
                                    label()
                                        .text(format!("Cape by {}", cape.username))
                                        .font_size(text_size(12.))
                                        .color(colors::fg_primary()),
                                )
                                .child(
                                    label()
                                        .text(format!("{} · free · 64×32 PNG", cape.updated_at))
                                        .font_size(text_size(10.))
                                        .color(colors::fg_secondary()),
                                ),
                        ),
                );
            }
        }

        content = content.child(section_heading("Publish your cape"));
        let selected_path = selected_cape_file.read().clone();
        let mut choose_file = selected_cape_file;
        let mut choose_status = cape_message;
        content = content.child(
            Button::new()
                .secondary()
                .size(ButtonSize::Small)
                .on_press(move |_| {
                    let mut selected = choose_file;
                    let mut status = choose_status;
                    spawn_global(async move {
                        if let Some(handle) = rfd::AsyncFileDialog::new()
                            .set_title("Choose a static 64×32 cape PNG")
                            .add_filter("PNG image", &["png"])
                            .pick_file()
                            .await
                        {
                            let path = handle.path().to_path_buf();
                            selected.set(Some(path.clone()));
                            status.set(Some(format!("Selected: {}", path.file_name().and_then(|name| name.to_str()).unwrap_or("cape.png"))));
                        }
                    });
                })
                .text("Choose cape PNG"),
        );

        if let Some(path) = selected_path.clone() {
            content = content.child(
                rect()
                    .horizontal()
                    .width(Size::fill())
                    .spacing(10.)
                    .cross_align(Alignment::Center)
                    .child(
                        rect()
                            .width(Size::px(68.))
                            .height(Size::px(52.))
                            .corner_radius(rounded(6.))
                            .overflow(Overflow::Clip)
                            .child(LocalImage::new(path.clone(), 128, false)),
                    )
                    .child(
                        label()
                            .text(path.file_name().and_then(|name| name.to_str()).unwrap_or("Selected cape.png"))
                            .font_size(text_size(11.))
                            .color(colors::fg_secondary())
                            .max_lines(1),
                    ),
            );
        }

        if let Some(active) = cape_session {
            let publish_token = active.access_token.clone();
            let publish_path = selected_path.clone();
            let mut publish_status = cape_message;
            let mut publish_busy = cape_busy;
            let mut publish_gallery = cape_gallery;
            content = content.child(
                Button::new()
                    .primary()
                    .size(ButtonSize::Large)
                    .width(Size::fill())
                    .disabled(publish_path.is_none() || *cape_busy.read())
                    .on_press(move |_| {
                        let Some(path) = publish_path.clone() else { return; };
                        let token = publish_token.clone();
                        let mut status = publish_status;
                        let mut busy = publish_busy;
                        let mut gallery = publish_gallery;
                        spawn_global(async move {
                            busy.set(true);
                            let result = async {
                                let bytes = tokio::fs::read(&path).await
                                    .map_err(|_| "Could not read the selected cape image.".to_string())?;
                                if bytes.len() > 256 * 1024 {
                                    return Err("Cape PNG must be 256 KiB or smaller.".to_string());
                                }
                                if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
                                    return Err("Please select a valid PNG file.".to_string());
                                }
                                let decoded = image::load_from_memory(&bytes)
                                    .map_err(|_| "The selected file is not a valid PNG image.".to_string())?;
                                if decoded.width() != 64 || decoded.height() != 32 {
                                    return Err("Cape images must be exactly 64×32 pixels.".to_string());
                                }
                                orange_api::publish_cape(&token, bytes).await?;
                                Ok::<(), String>(())
                            }.await;
                            match result {
                                Ok(()) => {
                                    status.set(Some("Cape published. This replaces your previous cape.".to_string()));
                                    gallery.set(Vec::new());
                                }
                                Err(error) => status.set(Some(error)),
                            }
                            busy.set(false);
                        });
                    })
                    .text("Publish / replace cape"),
            );
        } else {
            content = content.child(
                label()
                    .text("Connect Orange Friends with your Microsoft Minecraft account above to publish. Community capes can still be browsed.")
                    .font_size(text_size(11.))
                    .color(colors::fg_secondary()),
            );
        }

        if let Some(cape_message) = cape_message.read().clone() {
            content = content.child(
                label()
                    .text(cape_message)
                    .font_size(text_size(11.))
                    .color(colors::fg_secondary()),
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

        ScrollView::new()
            .width(Size::fill())
            .height(Size::fill())
            .show_scrollbar(true)
            .child(content)
            .into_element()
    }
}

fn section_heading(title: &'static str) -> Element {
    label()
        .text(title)
        .font_size(text_size(13.))
        .font_weight(FontWeight::SEMI_BOLD)
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
