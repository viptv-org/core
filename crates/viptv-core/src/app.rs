use crate::domain;
use crux_core::{
    App, Command,
    capability::Operation,
    macros::effect,
    render::{RenderOperation, render},
};
use crux_http::protocol::{HttpHeader, HttpRequest, HttpResult};
use facet::Facet;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct Session {
    pub session_id: String,
    pub account_id: String,
    pub profile_id: Option<String>,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct Profile {
    #[serde(default)]
    pub raw: crate::dto::JsonObject,
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub primary: Option<bool>,
    pub avatar_style: Option<String>,
    pub avatar_choice: Option<f64>,
    pub kid: Option<bool>,
    pub setup_complete: Option<bool>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
pub struct Account {
    pub id: String,
    pub username: String,
    pub name: String,
    pub role: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct Identity {
    pub account: Account,
    pub profiles: Vec<Profile>,
    pub profile_id: Option<String>,
    pub restricted: bool,
    pub profile_setup_required: bool,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize, Facet, PartialEq)]
#[repr(C)]
pub enum Phase {
    #[default]
    Starting,
    Restoring,
    Checking,
    Selecting,
    Ready,
    Profiles,
    Pairing,
    Error,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct ViewModel {
    pub phase: Phase,
    pub identity: Option<Identity>,
    pub selected_profile_id: Option<String>,
    pub error: Option<String>,
    pub error_status: Option<u16>,
}
#[derive(Serialize, Deserialize, Facet)]
#[repr(C)]
pub enum Event {
    Begin {
        origin: String,
        #[serde(rename = "allowInsecurePreview")]
        #[facet(rename = "allowInsecurePreview")]
        allow_insecure_preview: bool,
    },
    Retry,
    SelectProfile {
        #[serde(rename = "profileId")]
        #[facet(rename = "profileId")]
        profile_id: String,
    },
    AdoptSession {
        #[serde(rename = "tokensJson")]
        #[facet(rename = "tokensJson")]
        tokens_json: String,
    },
    SignOut,
    #[serde(skip)]
    #[facet(skip)]
    StorageCompleted(u64, StoragePurpose, StorageResult),
    #[serde(skip)]
    #[facet(skip)]
    HttpCompleted(u64, HttpPurpose, HttpResult),
}
#[derive(Clone, Serialize, Deserialize, Facet)]
#[repr(C)]
pub enum StorageOperation {
    Load,
    Save(String),
    Clear,
}
#[derive(Clone, Serialize, Deserialize, Facet)]
#[repr(C)]
pub enum StorageResult {
    Ok(Option<String>),
    Err(String),
}
impl Operation for StorageOperation {
    type Output = StorageResult;
}
#[effect(facet_typegen)]
pub enum Effect {
    Render(RenderOperation),
    Http(HttpRequest),
    Storage(StorageOperation),
}
#[derive(Clone, Copy, Facet)]
#[repr(C)]
pub enum StoragePurpose {
    Load,
    Refresh,
    Profile,
    Adopt,
    Clear,
}
#[derive(Clone, Facet)]
#[repr(C)]
pub enum HttpPurpose {
    Identity { refreshed: bool },
    Refresh,
    Select(String),
    Logout,
}
#[derive(Default)]
pub struct Model {
    pub view: ViewModel,
    origin: String,
    tokens: Option<Session>,
    epoch: u64,
    refresh_next: Option<HttpPurpose>,
    refresh_attempted: bool,
    // Bound the post-selection refresh: an inconsistent identity must not
    // silently repeat a successful profile mutation. Reset on external actions.
    accepted_profile: Option<String>,
}
#[derive(Default)]
pub struct Viptv;
fn storage(
    model: &Model,
    operation: StorageOperation,
    purpose: StoragePurpose,
) -> Command<Effect, Event> {
    let epoch = model.epoch;
    Command::request_from_shell(operation)
        .then_send(move |result| Event::StorageCompleted(epoch, purpose, result))
}
fn request(
    model: &Model,
    path: &str,
    method: &str,
    body: Option<serde_json::Value>,
    purpose: HttpPurpose,
) -> Command<Effect, Event> {
    let epoch = model.epoch;
    let mut headers = vec![HttpHeader {
        name: "Accept".into(),
        value: "application/json".into(),
    }];
    if let Some(tokens) = &model.tokens
        && !matches!(purpose, HttpPurpose::Refresh)
    {
        headers.push(HttpHeader {
            name: "Authorization".into(),
            value: format!("Bearer {}", tokens.access_token),
        });
    }
    if body.is_some() {
        headers.push(HttpHeader {
            name: "Content-Type".into(),
            value: "application/json".into(),
        });
    }
    let operation = HttpRequest {
        method: method.into(),
        url: format!("{}{path}", model.origin),
        headers,
        body: body.map(|b| b.to_string().into_bytes()).unwrap_or_default(),
    };
    Command::request_from_shell(operation)
        .then_send(move |result| Event::HttpCompleted(epoch, purpose, result))
}
fn me(model: &Model, refreshed: bool) -> Command<Effect, Event> {
    request(
        model,
        "/api/auth/me",
        "GET",
        None,
        HttpPurpose::Identity { refreshed },
    )
}
fn fail(model: &mut Model, message: &str) -> Command<Effect, Event> {
    model.view.phase = Phase::Error;
    model.view.error = Some(message.into());
    model.view.error_status = None;
    render()
}
fn save(model: &Model, purpose: StoragePurpose) -> Command<Effect, Event> {
    storage(
        model,
        StorageOperation::Save(
            serde_json::to_string(&model.tokens).expect("session is serializable"),
        ),
        purpose,
    )
}
fn select(model: &mut Model, id: String) -> Command<Effect, Event> {
    model.view.phase = Phase::Selecting;
    request(
        model,
        "/api/auth/profile",
        "POST",
        Some(json!({"profile_id":id})),
        HttpPurpose::Select(id),
    )
    .and(render())
}
fn finish_identity(model: &mut Model, identity: Identity) -> Command<Effect, Event> {
    let accepted_profile = model.accepted_profile.take();
    let server_profile = identity.profile_id.clone().filter(|id| {
        identity
            .profiles
            .iter()
            .any(|p| &p.id == id && p.setup_complete != Some(false))
    });
    let remembered = model
        .tokens
        .as_ref()
        .and_then(|s| s.profile_id.clone())
        .filter(|id| {
            identity
                .profiles
                .iter()
                .any(|p| &p.id == id && p.setup_complete != Some(false))
        });
    model.view.identity = Some(identity);
    if let Some(id) = server_profile {
        model.view.selected_profile_id = Some(id.clone());
        model.view.phase = Phase::Ready;
        if let Some(s) = &mut model.tokens
            && s.profile_id.as_ref() != Some(&id)
        {
            s.profile_id = Some(id);
            return save(model, StoragePurpose::Profile);
        }
        render()
    } else if accepted_profile.is_some() {
        model.view.selected_profile_id = None;
        if remembered.is_some() {
            fail(
                model,
                "The server did not confirm the selected profile. Please retry.",
            )
        } else {
            model.view.phase = Phase::Profiles;
            render()
        }
    } else if let Some(id) = remembered {
        select(model, id)
    } else {
        model.view.selected_profile_id = None;
        model.view.phase = Phase::Profiles;
        render()
    }
}
impl App for Viptv {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;
    fn view(&self, model: &Model) -> ViewModel {
        model.view.clone()
    }
    fn update(&self, event: Event, model: &mut Model) -> Command<Effect, Event> {
        match event {
            Event::Begin {
                origin,
                allow_insecure_preview,
            } => {
                let Ok(url) = url::Url::parse(&origin) else {
                    return fail(model, "Invalid server origin");
                };
                if (url.scheme() != "https" && !(allow_insecure_preview && url.scheme() == "http"))
                    || url.path() != "/"
                    || url.query().is_some()
                    || url.fragment().is_some()
                    || !url.username().is_empty()
                    || url.password().is_some()
                {
                    return fail(model, "Invalid server origin");
                }
                model.epoch += 1;
                model.accepted_profile = None;
                model.refresh_attempted = false;
                model.origin = url.origin().ascii_serialization();
                model.view = ViewModel {
                    phase: Phase::Restoring,
                    ..Default::default()
                };
                storage(model, StorageOperation::Load, StoragePurpose::Load).and(render())
            }
            Event::Retry => {
                model.epoch += 1;
                model.accepted_profile = None;
                model.refresh_attempted = false;
                model.view.error = None;
                model.view.error_status = None;
                if model.tokens.is_some() {
                    model.view.phase = Phase::Checking;
                    me(model, false).and(render())
                } else {
                    model.view.phase = Phase::Restoring;
                    storage(model, StorageOperation::Load, StoragePurpose::Load).and(render())
                }
            }
            Event::AdoptSession { tokens_json } => {
                let Ok(tokens) = serde_json::from_str::<Session>(&tokens_json) else {
                    return fail(model, "Invalid saved session");
                };
                if tokens.access_token.is_empty() || tokens.refresh_token.is_empty() {
                    return fail(model, "Invalid saved session");
                }
                model.epoch += 1;
                model.accepted_profile = None;
                model.refresh_attempted = false;
                model.tokens = Some(tokens);
                model.view = ViewModel {
                    phase: Phase::Checking,
                    ..Default::default()
                };
                save(model, StoragePurpose::Adopt).and(render())
            }
            Event::SelectProfile { profile_id } => {
                if !matches!(
                    model.view.phase,
                    Phase::Profiles | Phase::Ready | Phase::Error
                ) || !model
                    .view
                    .identity
                    .as_ref()
                    .is_some_and(|i| i.profiles.iter().any(|p| p.id == profile_id))
                {
                    return Command::done();
                }
                model.epoch += 1;
                model.accepted_profile = None;
                model.refresh_attempted = false;
                model.view.error = None;
                model.view.error_status = None;
                select(model, profile_id)
            }
            Event::SignOut => {
                model.view.error = None;
                model.view.error_status = None;
                if model.tokens.is_none() {
                    return Command::done();
                }
                model.epoch += 1;
                model.accepted_profile = None;
                model.refresh_attempted = false;
                request(
                    model,
                    "/api/auth/logout",
                    "POST",
                    Some(json!({})),
                    HttpPurpose::Logout,
                )
            }
            Event::StorageCompleted(epoch, purpose, result) => {
                if epoch != model.epoch {
                    return Command::done();
                }
                let StorageResult::Ok(value) = result else {
                    return fail(
                        model,
                        "Could not access saved session. Retry to stay paired.",
                    );
                };
                match purpose {
                    StoragePurpose::Load => match value {
                        None => {
                            model.tokens = None;
                            model.view.phase = Phase::Pairing;
                            render()
                        }
                        Some(value) => {
                            let Ok(tokens) = serde_json::from_str::<Session>(&value) else {
                                return fail(model, "Saved session could not be read");
                            };
                            model.tokens = Some(tokens);
                            model.view.phase = Phase::Checking;
                            me(model, false).and(render())
                        }
                    },
                    StoragePurpose::Refresh => {
                        model.view.phase = Phase::Checking;
                        match model.refresh_next.take() {
                            Some(HttpPurpose::Select(id)) => select(model, id),
                            Some(HttpPurpose::Logout) => request(
                                model,
                                "/api/auth/logout",
                                "POST",
                                Some(json!({})),
                                HttpPurpose::Logout,
                            ),
                            _ => me(model, true),
                        }
                    }
                    StoragePurpose::Adopt => me(model, false),
                    StoragePurpose::Profile => me(model, true),
                    StoragePurpose::Clear => {
                        model.tokens = None;
                        model.view = ViewModel {
                            phase: Phase::Pairing,
                            ..Default::default()
                        };
                        render()
                    }
                }
            }
            Event::HttpCompleted(epoch, purpose, result) => {
                if epoch != model.epoch {
                    return Command::done();
                }
                let HttpResult::Ok(response) = result else {
                    return fail(model, "Unable to connect. Your saved session is retained.");
                };
                if response.status == 401 {
                    if matches!(
                        purpose,
                        HttpPurpose::Refresh | HttpPurpose::Identity { refreshed: true }
                    ) || model.refresh_attempted
                    {
                        return storage(model, StorageOperation::Clear, StoragePurpose::Clear);
                    }
                    let Some(tokens) = &model.tokens else {
                        model.view.phase = Phase::Pairing;
                        return render();
                    };
                    model.refresh_attempted = true;
                    model.refresh_next = Some(purpose);
                    return request(
                        model,
                        "/api/auth/device/refresh",
                        "POST",
                        Some(json!({"refresh_token":tokens.refresh_token})),
                        HttpPurpose::Refresh,
                    );
                }
                if !(200..300).contains(&response.status) {
                    let command = fail(
                        model,
                        match response.status {
                            403 => "This action needs parent authorization",
                            429 => "Please try again shortly",
                            _ => "The server could not complete the request",
                        },
                    );
                    model.view.error_status = Some(response.status);
                    return command;
                }
                if response.body.len() > 2 * 1024 * 1024 {
                    return fail(model, "Invalid server response");
                }
                let payload = if response.body.is_empty() {
                    json!({})
                } else {
                    match serde_json::from_slice(&response.body) {
                        Ok(v) => v,
                        Err(_) => return fail(model, "Invalid server response"),
                    }
                };
                match purpose {
                    HttpPurpose::Identity { .. } => {
                        let identity = domain::identity(&payload).and_then(|v| {
                            serde_json::from_value(v).map_err(|_| crate::CoreError::InvalidInput)
                        });
                        match identity {
                            Ok(identity) => finish_identity(model, identity),
                            Err(_) => fail(model, "Invalid server response"),
                        }
                    }
                    HttpPurpose::Refresh => {
                        let session = domain::tokens(&payload).and_then(|v| {
                            serde_json::from_value::<Session>(v)
                                .map_err(|_| crate::CoreError::InvalidInput)
                        });
                        match session {
                            Ok(mut session) => {
                                if session.profile_id.is_none()
                                    && model
                                        .tokens
                                        .as_ref()
                                        .is_some_and(|old| old.account_id == session.account_id)
                                {
                                    session.profile_id =
                                        model.tokens.as_ref().and_then(|s| s.profile_id.clone());
                                }
                                model.tokens = Some(session);
                                save(model, StoragePurpose::Refresh)
                            }
                            Err(_) => fail(model, "Invalid server response"),
                        }
                    }
                    HttpPurpose::Select(id) => {
                        model.accepted_profile = Some(id.clone());
                        if let Some(tokens) = &mut model.tokens {
                            tokens.profile_id = Some(id);
                        }
                        save(model, StoragePurpose::Profile)
                    }
                    HttpPurpose::Logout => {
                        storage(model, StorageOperation::Clear, StoragePurpose::Clear)
                    }
                }
            }
        }
    }
}
