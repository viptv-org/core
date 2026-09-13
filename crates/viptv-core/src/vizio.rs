//! Portable Vizio SmartCast protocol behavior.
//!
//! Ported from get-air/vizio at 124b5fb8f2b2b04b3fb9237d4c72b1c4e1a19e3d.
//! Copyright (c) 2026 Air contributors; used under the MIT license recorded in
//! THIRD_PARTY_LICENSES/get-air-vizio.txt. Network/TLS and credential storage
//! remain platform adapters; this module never relaxes certificate validation.

use crate::dto::{JsonObject, JsonValue};
use facet::Facet;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

const SETTINGS_ROOT: &str = "/menu_native/dynamic/tv_settings";
const DEFAULT_DEVICE_ID: &str = "viptv-smartcast";
const DEFAULT_DEVICE_NAME: &str = "VIPTV";

#[derive(Clone, Debug, Serialize, Deserialize, Facet, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
#[facet(rename_all = "UPPERCASE")]
#[repr(C)]
pub enum VizioHttpMethod {
    Get,
    Put,
}

#[derive(Clone, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct VizioRequest {
    pub method: VizioHttpMethod,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub body: Option<JsonObject>,
    pub timeout_millis: u64,
    pub max_response_bytes: u64,
}

impl std::fmt::Debug for VizioRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VizioRequest")
            .field("method", &self.method)
            .field("url", &"<redacted>")
            .field("headers", &"<redacted>")
            .field("body", &self.body.as_ref().map(|_| "<redacted>"))
            .field("timeout_millis", &self.timeout_millis)
            .field("max_response_bytes", &self.max_response_bytes)
            .finish()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Facet, PartialEq)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
#[repr(C)]
pub enum VizioFailureKind {
    InvalidConfig,
    InvalidInput,
    Authentication,
    InvalidParameter,
    EndpointNotFound,
    Busy,
    Transport,
    InvalidResponse,
    HttpStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct VizioFailure {
    pub kind: VizioFailureKind,
    pub message: String,
    pub retryable: bool,
    pub protocol_status: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Facet)]
#[repr(C)]
pub enum VizioRequestResult {
    Ok(VizioRequest),
    Err(VizioFailure),
}

#[derive(Clone, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct VizioProtocolResponse {
    pub status: String,
    pub detail: String,
    pub raw: JsonObject,
}

impl std::fmt::Debug for VizioProtocolResponse {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VizioProtocolResponse")
            .field("status", &self.status)
            .field("detail", &"<redacted>")
            .field("raw", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[repr(C)]
pub enum VizioResponseResult {
    Ok(VizioProtocolResponse),
    Err(VizioFailure),
}

#[derive(Clone, Debug, Serialize, Deserialize, Facet, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[facet(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(C)]
pub enum VizioRemoteAction {
    Keypress,
    Keydown,
    Keyup,
}

#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct VizioRemoteEvent {
    pub code_set: u16,
    pub code: u16,
    pub action: VizioRemoteAction,
}

#[derive(Clone, Debug, Serialize, Deserialize, Facet, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[facet(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(C)]
pub enum VizioRemoteKey {
    SeekFwd,
    SeekBack,
    Pause,
    Play,
    Down,
    Left,
    Ok,
    Right,
    Up,
    Back,
    Smartcast,
    CcToggle,
    Info,
    Menu,
    Home,
    VolDown,
    VolUp,
    MuteOff,
    MuteOn,
    MuteToggle,
    PicMode,
    PicSize,
    InputNext,
    ChDown,
    ChUp,
    ChPrev,
    Exit,
    PowOff,
    PowOn,
    PowToggle,
}

impl VizioRemoteKey {
    pub fn event(&self, action: VizioRemoteAction) -> VizioRemoteEvent {
        let (code_set, code) = match self {
            Self::SeekFwd => (2, 0),
            Self::SeekBack => (2, 1),
            Self::Pause => (2, 2),
            Self::Play => (2, 3),
            Self::Down => (3, 0),
            Self::Left => (3, 1),
            Self::Ok => (3, 2),
            Self::Right => (3, 7),
            Self::Up => (3, 8),
            Self::Back => (4, 0),
            Self::Smartcast => (4, 3),
            Self::CcToggle => (4, 4),
            Self::Info => (4, 6),
            Self::Menu => (4, 8),
            Self::Home => (4, 15),
            Self::VolDown => (5, 0),
            Self::VolUp => (5, 1),
            Self::MuteOff => (5, 2),
            Self::MuteOn => (5, 3),
            Self::MuteToggle => (5, 4),
            Self::PicMode => (6, 0),
            Self::PicSize => (6, 2),
            Self::InputNext => (7, 1),
            Self::ChDown => (8, 0),
            Self::ChUp => (8, 1),
            Self::ChPrev => (8, 2),
            Self::Exit => (9, 0),
            Self::PowOff => (11, 0),
            Self::PowOn => (11, 1),
            Self::PowToggle => (11, 2),
        };
        VizioRemoteEvent {
            code_set,
            code,
            action,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct VizioPairingChallenge {
    pub challenge_type: u16,
    pub token: u64,
}

impl std::fmt::Debug for VizioPairingChallenge {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VizioPairingChallenge")
            .field("challenge_type", &self.challenge_type)
            .field("token", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct VizioAppConfig {
    pub app_id: String,
    pub name_space: u16,
    pub message: Option<String>,
}

impl std::fmt::Debug for VizioAppConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VizioAppConfig")
            .field("app_id", &self.app_id)
            .field("name_space", &self.name_space)
            .field("message", &self.message.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct VizioInputInfo {
    pub cname: String,
    pub name: String,
    pub meta_name: String,
    pub current: bool,
    pub hash_value: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct VizioDiscoveryCandidate {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, Facet, PartialEq)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
#[repr(C)]
pub enum VizioTransportSupport {
    Native,
    Unavailable,
}

#[derive(Clone, Debug, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct VizioPlatformSupport {
    pub protocol_available: bool,
    pub transport: VizioTransportSupport,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Facet, PartialEq)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
#[repr(C)]
pub enum VizioControllerOutputKind {
    Request,
    Complete,
    Error,
}

#[derive(Clone, Serialize, Deserialize, Facet)]
#[serde(rename_all = "camelCase")]
#[facet(rename_all = "camelCase")]
pub struct VizioControllerOutput {
    pub kind: VizioControllerOutputKind,
    pub request_id: Option<u32>,
    pub request: Option<VizioRequest>,
    pub result: Option<JsonObject>,
    pub error: Option<VizioFailure>,
    pub credential_changed: bool,
}

impl std::fmt::Debug for VizioControllerOutput {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VizioControllerOutput")
            .field("kind", &self.kind)
            .field("request_id", &self.request_id)
            .field("request", &self.request.as_ref().map(|_| "<redacted>"))
            .field("result", &self.result.as_ref().map(|_| "<redacted>"))
            .field("error", &self.error)
            .field("credential_changed", &self.credential_changed)
            .finish()
    }
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ControllerConfig {
    host: String,
    auth_token: Option<String>,
    device_id: Option<String>,
    device_name: Option<String>,
    timeout_millis: Option<u64>,
    max_response_bytes: Option<u64>,
}

#[derive(Clone)]
enum SettingMutation {
    Modify(Value),
    Action,
}

enum PendingWorkflow {
    Direct,
    BeginPair,
    FinishPair,
    InputsCurrent {
        wanted: Option<String>,
    },
    InputsList {
        wanted: Option<String>,
        current: String,
    },
    InputFresh {
        cname: String,
    },
    InputWrite,
    SettingRead {
        category: String,
        name: String,
        mutation: SettingMutation,
        retried: bool,
    },
    SettingWrite {
        category: String,
        name: String,
        mutation: SettingMutation,
        retried: bool,
    },
}

pub struct VizioController {
    config: ControllerConfig,
    next_request_id: u32,
    pending: Option<(u32, PendingWorkflow)>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestInput {
    host: String,
    auth_token: Option<String>,
    device_id: Option<String>,
    device_name: Option<String>,
    timeout_millis: Option<u64>,
    max_response_bytes: Option<u64>,
    #[serde(flatten)]
    fields: Map<String, Value>,
}

struct RequestLimits {
    timeout_millis: u64,
    max_response_bytes: u64,
}

pub fn plan_request(operation: &str, input: &str) -> VizioRequestResult {
    let result = serde_json::from_str::<RequestInput>(input)
        .map_err(|_| {
            failure(
                VizioFailureKind::InvalidInput,
                "Invalid SmartCast command",
                false,
            )
        })
        .and_then(|input| request_for(operation, input));
    match result {
        Ok(request) => VizioRequestResult::Ok(request),
        Err(error) => VizioRequestResult::Err(error),
    }
}

pub fn parse_response(status: u16, body: &str, allow_statusless: bool) -> VizioResponseResult {
    let result = parse_protocol_response(status, body, allow_statusless);
    match result {
        Ok(response) => VizioResponseResult::Ok(response),
        Err(error) => VizioResponseResult::Err(error),
    }
}

impl VizioController {
    pub fn new(config: &str) -> Result<Self, VizioFailure> {
        if config.len() > 16 * 1024 {
            return Err(failure(
                VizioFailureKind::InvalidConfig,
                "SmartCast configuration is too large",
                false,
            ));
        }
        let mut config = serde_json::from_str::<ControllerConfig>(config).map_err(|_| {
            failure(
                VizioFailureKind::InvalidConfig,
                "Invalid SmartCast configuration",
                false,
            )
        })?;
        config.host = normalize_host(&config.host)?;
        validate_identity(config.device_id.as_deref(), "device ID")?;
        validate_identity(config.device_name.as_deref(), "device name")?;
        validate_limits(config.timeout_millis, config.max_response_bytes)?;
        if config.auth_token.as_ref().is_some_and(|token| {
            token.is_empty() || token.len() > 1024 || token.chars().any(char::is_control)
        }) {
            return Err(failure(
                VizioFailureKind::InvalidConfig,
                "Invalid SmartCast credential",
                false,
            ));
        }
        Ok(Self {
            config,
            next_request_id: 1,
            pending: None,
        })
    }

    pub fn start(&mut self, operation: &str, input: &str) -> VizioControllerOutput {
        if self.pending.is_some() {
            return controller_error(failure(
                VizioFailureKind::Busy,
                "A SmartCast command is already running",
                true,
            ));
        }
        let fields = match command_fields(input) {
            Ok(fields) => fields,
            Err(error) => return controller_error(error),
        };
        match operation {
            "beginPair" => self.enqueue("beginPair", fields, PendingWorkflow::BeginPair),
            "finishPair" => self.enqueue("finishPair", fields, PendingWorkflow::FinishPair),
            "getInputs" => self.enqueue(
                "getCurrentInput",
                Map::new(),
                PendingWorkflow::InputsCurrent { wanted: None },
            ),
            "setInput" => {
                let wanted = match required_string(&fields, "input") {
                    Ok(value) if !value.trim().is_empty() && value.len() <= 128 => value.to_owned(),
                    _ => {
                        return controller_error(failure(
                            VizioFailureKind::InvalidInput,
                            "TV input is required",
                            false,
                        ));
                    }
                };
                self.enqueue(
                    "getCurrentInput",
                    Map::new(),
                    PendingWorkflow::InputsCurrent {
                        wanted: Some(wanted),
                    },
                )
            }
            "setSetting" => {
                let (category, name) = match setting_names(&fields) {
                    Ok(names) => names,
                    Err(error) => return controller_error(error),
                };
                let Some(value) = fields.get("value").cloned() else {
                    return controller_error(failure(
                        VizioFailureKind::InvalidInput,
                        "Setting value is required",
                        false,
                    ));
                };
                if !is_setting_value(&value) {
                    return controller_error(failure(
                        VizioFailureKind::InvalidParameter,
                        "Setting value must be a string, number, or boolean",
                        false,
                    ));
                }
                self.enqueue(
                    "getSetting",
                    setting_fields(&category, &name),
                    PendingWorkflow::SettingRead {
                        category,
                        name,
                        mutation: SettingMutation::Modify(value),
                        retried: false,
                    },
                )
            }
            "triggerSetting" | "blankScreen" => {
                let names = if operation == "blankScreen" {
                    Ok(("system/timers".to_owned(), "blank_screen".to_owned()))
                } else {
                    setting_names(&fields)
                };
                let (category, name) = match names {
                    Ok(names) => names,
                    Err(error) => return controller_error(error),
                };
                self.enqueue(
                    "getSetting",
                    setting_fields(&category, &name),
                    PendingWorkflow::SettingRead {
                        category,
                        name,
                        mutation: SettingMutation::Action,
                        retried: false,
                    },
                )
            }
            _ => self.enqueue(operation, fields, PendingWorkflow::Direct),
        }
    }

    pub fn resolve(&mut self, request_id: u32, status: u16, body: &str) -> VizioControllerOutput {
        let Some((expected, _)) = self.pending.as_ref() else {
            return controller_error(failure(
                VizioFailureKind::InvalidInput,
                "No SmartCast request is pending",
                false,
            ));
        };
        if *expected != request_id {
            return controller_error(failure(
                VizioFailureKind::InvalidInput,
                "Stale SmartCast response was ignored",
                false,
            ));
        }
        let (_, pending) = self.pending.take().expect("pending request checked above");
        let response = match parse_protocol_response(status, body, false) {
            Ok(response) => response,
            Err(error) => {
                if error.protocol_status.as_deref() == Some("HASHVAL_ERROR")
                    && let PendingWorkflow::SettingWrite {
                        category,
                        name,
                        mutation,
                        retried: false,
                    } = pending
                {
                    return self.enqueue(
                        "getSetting",
                        setting_fields(&category, &name),
                        PendingWorkflow::SettingRead {
                            category,
                            name,
                            mutation,
                            retried: true,
                        },
                    );
                }
                return controller_error(error);
            }
        };
        match pending {
            PendingWorkflow::Direct => controller_complete(Some(response.raw), false),
            PendingWorkflow::BeginPair => match pairing_challenge(&response) {
                Ok(challenge) => controller_complete(
                    object_value(json!({
                        "challengeType": challenge.challenge_type,
                        "token": challenge.token,
                    })),
                    false,
                ),
                Err(error) => controller_error(error),
            },
            PendingWorkflow::FinishPair => match pairing_auth_token(&response) {
                Ok(token) => {
                    self.config.auth_token = Some(token);
                    controller_complete(object_value(json!({"paired": true})), true)
                }
                Err(error) => controller_error(error),
            },
            PendingWorkflow::InputsCurrent { wanted } => match current_input_state(&response) {
                Ok((current, _)) => self.enqueue(
                    "getInputs",
                    Map::new(),
                    PendingWorkflow::InputsList { wanted, current },
                ),
                Err(error) => controller_error(error),
            },
            PendingWorkflow::InputsList { wanted, current } => {
                let inputs = parse_inputs(&response, &current);
                if let Some(wanted) = wanted {
                    let target = match match_input(&wanted, &inputs) {
                        Ok(target) => target,
                        Err(error) => return controller_error(error),
                    };
                    if target.current {
                        return controller_complete(
                            object_value(json!({"changed": false, "input": target.cname})),
                            false,
                        );
                    }
                    self.enqueue(
                        "getCurrentInput",
                        Map::new(),
                        PendingWorkflow::InputFresh {
                            cname: target.cname.clone(),
                        },
                    )
                } else {
                    controller_complete(object_value(json!({"inputs": inputs})), false)
                }
            }
            PendingWorkflow::InputFresh { cname } => match current_input_state(&response) {
                Ok((_, hash_value)) => {
                    let mut fields = Map::new();
                    fields.insert("cname".into(), Value::String(cname));
                    fields.insert("hashValue".into(), Value::from(hash_value));
                    self.enqueue("setInput", fields, PendingWorkflow::InputWrite)
                }
                Err(error) => controller_error(error),
            },
            PendingWorkflow::InputWrite => {
                controller_complete(object_value(json!({"changed": true})), false)
            }
            PendingWorkflow::SettingRead {
                category,
                name,
                mutation,
                retried,
            } => {
                let setting = match parse_setting(&response, &name) {
                    Ok(setting) => setting,
                    Err(error) => return controller_error(error),
                };
                if let SettingMutation::Modify(value) = &mutation
                    && let Err(error) = validate_setting_snapshot(value, &setting)
                {
                    return controller_error(error);
                }
                let mut fields = setting_fields(&category, &name);
                fields.insert("hashValue".into(), Value::from(setting.hash_value));
                let operation = match &mutation {
                    SettingMutation::Modify(value) => {
                        fields.insert("value".into(), value.clone());
                        "setSetting"
                    }
                    SettingMutation::Action => "triggerSetting",
                };
                self.enqueue(
                    operation,
                    fields,
                    PendingWorkflow::SettingWrite {
                        category,
                        name,
                        mutation,
                        retried,
                    },
                )
            }
            PendingWorkflow::SettingWrite { retried, .. } => controller_complete(
                object_value(json!({"changed": true, "retried": retried})),
                false,
            ),
        }
    }

    pub fn reject(&mut self, request_id: u32) -> VizioControllerOutput {
        if self
            .pending
            .as_ref()
            .is_some_and(|(expected, _)| *expected == request_id)
        {
            self.pending = None;
            controller_error(failure(
                VizioFailureKind::Transport,
                "SmartCast transport failed",
                true,
            ))
        } else {
            controller_error(failure(
                VizioFailureKind::InvalidInput,
                "Stale SmartCast response was ignored",
                false,
            ))
        }
    }

    pub fn cancel(&mut self) {
        self.pending = None;
    }

    pub fn auth_token(&self) -> Option<String> {
        self.config.auth_token.clone()
    }

    pub fn clear_auth_token(&mut self) {
        self.config.auth_token = None;
        self.pending = None;
    }

    fn enqueue(
        &mut self,
        operation: &str,
        fields: Map<String, Value>,
        pending: PendingWorkflow,
    ) -> VizioControllerOutput {
        let input = RequestInput {
            host: self.config.host.clone(),
            auth_token: self.config.auth_token.clone(),
            device_id: self.config.device_id.clone(),
            device_name: self.config.device_name.clone(),
            timeout_millis: self.config.timeout_millis,
            max_response_bytes: self.config.max_response_bytes,
            fields,
        };
        match request_for(operation, input) {
            Ok(request) => {
                let request_id = self.next_request_id;
                self.next_request_id = self.next_request_id.wrapping_add(1).max(1);
                self.pending = Some((request_id, pending));
                VizioControllerOutput {
                    kind: VizioControllerOutputKind::Request,
                    request_id: Some(request_id),
                    request: Some(request),
                    result: None,
                    error: None,
                    credential_changed: false,
                }
            }
            Err(error) => controller_error(error),
        }
    }
}

pub fn discovery_candidates(subnet: &str) -> Result<Vec<VizioDiscoveryCandidate>, VizioFailure> {
    let normalized = subnet.trim().trim_end_matches('.');
    let octets = normalized
        .split('.')
        .map(str::parse::<u8>)
        .collect::<Result<Vec<_>, _>>();
    let Ok(octets) = octets else {
        return Err(failure(
            VizioFailureKind::InvalidConfig,
            "Subnet must be an IPv4 /24 prefix such as 192.168.1",
            false,
        ));
    };
    if octets.len() != 3 {
        return Err(failure(
            VizioFailureKind::InvalidConfig,
            "Subnet must be an IPv4 /24 prefix such as 192.168.1",
            false,
        ));
    }
    let normalized = octets
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(".");
    Ok((1..=254)
        .flat_map(|host| {
            [7345, 9000].map(|port| VizioDiscoveryCandidate {
                host: format!("{normalized}.{host}:{port}"),
                port,
            })
        })
        .collect())
}

pub fn platform_support(platform: &str) -> VizioPlatformSupport {
    match platform.trim().to_ascii_lowercase().as_str() {
        "android" | "android-mobile" | "tauri" | "desktop" => VizioPlatformSupport {
            protocol_available: true,
            transport: VizioTransportSupport::Native,
            reason: "Use a native exact-TV-origin HTTPS adapter and OS credential vault.".into(),
        },
        _ => VizioPlatformSupport {
            protocol_available: false,
            transport: VizioTransportSupport::Unavailable,
            reason: "SmartCast delivery is scoped to Android mobile and Tauri desktop.".into(),
        },
    }
}

pub fn pairing_challenge(
    response: &VizioProtocolResponse,
) -> Result<VizioPairingChallenge, VizioFailure> {
    let item = response_item(&response.raw).ok_or_else(|| {
        failure(
            VizioFailureKind::InvalidResponse,
            "Pairing response is missing challenge data",
            false,
        )
    })?;
    let challenge_type = number(item, "CHALLENGE_TYPE").and_then(number_to_u16);
    let token = number(item, "PAIRING_REQ_TOKEN")
        .and_then(number_to_u32)
        .map(u64::from);
    match (challenge_type, token) {
        (Some(challenge_type), Some(token)) => Ok(VizioPairingChallenge {
            challenge_type,
            token,
        }),
        _ => Err(failure(
            VizioFailureKind::InvalidResponse,
            "Pairing response is missing challenge data",
            false,
        )),
    }
}

pub fn pairing_auth_token(response: &VizioProtocolResponse) -> Result<String, VizioFailure> {
    response_item(&response.raw)
        .and_then(|item| string(item, "AUTH_TOKEN"))
        .filter(|token| {
            !token.trim().is_empty() && token.len() <= 1024 && !token.chars().any(char::is_control)
        })
        .map(str::to_owned)
        .ok_or_else(|| {
            failure(
                VizioFailureKind::InvalidResponse,
                "Pairing response is missing AUTH_TOKEN",
                false,
            )
        })
}

struct SettingSnapshot {
    hash_value: i64,
    minimum: Option<f64>,
    maximum: Option<f64>,
    options: Vec<String>,
}

fn command_fields(input: &str) -> Result<Map<String, Value>, VizioFailure> {
    if input.len() > 64 * 1024 {
        return Err(failure(
            VizioFailureKind::InvalidInput,
            "SmartCast command is too large",
            false,
        ));
    }
    let value = if input.trim().is_empty() {
        Value::Object(Map::new())
    } else {
        serde_json::from_str::<Value>(input).map_err(|_| {
            failure(
                VizioFailureKind::InvalidInput,
                "Invalid SmartCast command",
                false,
            )
        })?
    };
    let fields = value.as_object().cloned().ok_or_else(|| {
        failure(
            VizioFailureKind::InvalidInput,
            "SmartCast command must be an object",
            false,
        )
    })?;
    if fields.keys().any(|key| {
        [
            "host",
            "authToken",
            "deviceId",
            "deviceName",
            "timeoutMillis",
            "maxResponseBytes",
            "hashValue",
            "cname",
        ]
        .contains(&key.as_str())
    }) {
        return Err(failure(
            VizioFailureKind::InvalidInput,
            "SmartCast command contains controller-owned data",
            false,
        ));
    }
    Ok(fields)
}

fn current_input_state(response: &VizioProtocolResponse) -> Result<(String, i64), VizioFailure> {
    let item = find_response_item(response, "current_input").ok_or_else(|| {
        failure(
            VizioFailureKind::InvalidResponse,
            "Current input response is missing data",
            false,
        )
    })?;
    let value = string(item, "VALUE").filter(|value| !value.is_empty());
    let hash_value = number(item, "HASHVAL").and_then(number_to_i64);
    match (value, hash_value) {
        (Some(value), Some(hash_value)) => Ok((value.to_owned(), hash_value)),
        _ => Err(failure(
            VizioFailureKind::InvalidResponse,
            "Current input response is missing VALUE or HASHVAL",
            false,
        )),
    }
}

fn parse_setting(
    response: &VizioProtocolResponse,
    name: &str,
) -> Result<SettingSnapshot, VizioFailure> {
    let item = find_response_item(response, name).ok_or_else(|| {
        failure(
            VizioFailureKind::InvalidResponse,
            "Setting response is missing data",
            false,
        )
    })?;
    let hash_value = number(item, "HASHVAL")
        .and_then(number_to_i64)
        .ok_or_else(|| {
            failure(
                VizioFailureKind::InvalidResponse,
                "Setting response is missing HASHVAL",
                false,
            )
        })?;
    let options = item
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("ELEMENTS"))
        .and_then(|(_, value)| match value {
            JsonValue::Array(values) => Some(values),
            _ => None,
        })
        .into_iter()
        .flatten()
        .filter_map(|entry| match entry {
            JsonValue::String(value) => Some(value.clone()),
            JsonValue::Object(value) => string(value, "NAME")
                .or_else(|| string(value, "VALUE"))
                .map(str::to_owned),
            _ => None,
        })
        .collect();
    Ok(SettingSnapshot {
        hash_value,
        minimum: number(item, "MINIMUM"),
        maximum: number(item, "MAXIMUM"),
        options,
    })
}

fn find_response_item<'a>(
    response: &'a VizioProtocolResponse,
    name: &str,
) -> Option<&'a JsonObject> {
    response_item(&response.raw)
        .into_iter()
        .chain(response_items(&response.raw))
        .find(|item| string(item, "CNAME").is_some_and(|value| value.eq_ignore_ascii_case(name)))
        .or_else(|| response_item(&response.raw))
        .or_else(|| response_items(&response.raw).into_iter().next())
}

fn validate_setting_snapshot(value: &Value, setting: &SettingSnapshot) -> Result<(), VizioFailure> {
    if !is_setting_value(value) {
        return Err(failure(
            VizioFailureKind::InvalidParameter,
            "Setting value must be a string, number, or boolean",
            false,
        ));
    }
    let mut fields = Map::new();
    if let Some(minimum) = setting.minimum {
        fields.insert("minimum".into(), Value::from(minimum));
    }
    if let Some(maximum) = setting.maximum {
        fields.insert("maximum".into(), Value::from(maximum));
    }
    if !setting.options.is_empty() {
        fields.insert("options".into(), json!(setting.options));
    }
    validate_setting_value(value, &fields)
}

fn is_setting_value(value: &Value) -> bool {
    value.is_string() || value.is_boolean() || value.is_number()
}

fn setting_names(fields: &Map<String, Value>) -> Result<(String, String), VizioFailure> {
    let category = setting_path(required_string(fields, "category")?)?.to_owned();
    let name = setting_path(required_string(fields, "name")?)?.to_owned();
    Ok((category, name))
}

fn setting_fields(category: &str, name: &str) -> Map<String, Value> {
    Map::from_iter([
        ("category".into(), Value::String(category.to_owned())),
        ("name".into(), Value::String(name.to_owned())),
    ])
}

fn validate_identity(value: Option<&str>, label: &str) -> Result<(), VizioFailure> {
    if value.is_some_and(|value| {
        value.is_empty() || value.len() > 128 || value.chars().any(char::is_control)
    }) {
        return Err(failure(
            VizioFailureKind::InvalidConfig,
            &format!("Invalid SmartCast {label}"),
            false,
        ));
    }
    Ok(())
}

fn validate_limits(timeout: Option<u64>, response_bytes: Option<u64>) -> Result<(), VizioFailure> {
    let timeout = timeout.unwrap_or(10_000);
    let response_bytes = response_bytes.unwrap_or(2 * 1024 * 1024);
    if !(250..=60_000).contains(&timeout) || response_bytes == 0 || response_bytes > 8 * 1024 * 1024
    {
        return Err(failure(
            VizioFailureKind::InvalidConfig,
            "Invalid SmartCast request limits",
            false,
        ));
    }
    Ok(())
}

fn controller_error(error: VizioFailure) -> VizioControllerOutput {
    VizioControllerOutput {
        kind: VizioControllerOutputKind::Error,
        request_id: None,
        request: None,
        result: None,
        error: Some(error),
        credential_changed: false,
    }
}

fn controller_complete(
    result: Option<JsonObject>,
    credential_changed: bool,
) -> VizioControllerOutput {
    VizioControllerOutput {
        kind: VizioControllerOutputKind::Complete,
        request_id: None,
        request: None,
        result,
        error: None,
        credential_changed,
    }
}

fn object_value(value: Value) -> Option<JsonObject> {
    value_object(value).ok()
}

pub fn parse_inputs(response: &VizioProtocolResponse, current: &str) -> Vec<VizioInputInfo> {
    response_items(&response.raw)
        .into_iter()
        .filter_map(|item| {
            let cname = string(item, "CNAME")?.to_owned();
            let name = string(item, "NAME")?.to_owned();
            let metadata = record(item, "VALUE");
            let meta_name = metadata
                .and_then(|value| string(value, "NAME").or_else(|| string(value, "METADATA")))
                .unwrap_or_default()
                .to_owned();
            let is_current = [&cname, &name, &meta_name]
                .iter()
                .any(|value| !value.is_empty() && value.eq_ignore_ascii_case(current));
            Some(VizioInputInfo {
                cname,
                name,
                meta_name,
                current: is_current,
                hash_value: number(item, "HASHVAL").map(|value| value as i64),
            })
        })
        .collect()
}

pub fn match_input<'a>(
    wanted: &str,
    inputs: &'a [VizioInputInfo],
) -> Result<&'a VizioInputInfo, VizioFailure> {
    let wanted = wanted.trim();
    let matches = inputs
        .iter()
        .filter(|candidate| {
            [&candidate.cname, &candidate.name, &candidate.meta_name]
                .iter()
                .any(|value| !value.is_empty() && value.eq_ignore_ascii_case(wanted))
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [single] => Ok(single),
        [] => Err(failure(
            VizioFailureKind::InvalidInput,
            "Unknown TV input",
            false,
        )),
        _ => Err(failure(
            VizioFailureKind::InvalidInput,
            "TV input is ambiguous",
            false,
        )),
    }
}

fn request_for(operation: &str, input: RequestInput) -> Result<VizioRequest, VizioFailure> {
    let base = normalize_host(&input.host)?;
    let auth = input
        .auth_token
        .as_deref()
        .filter(|token| !token.is_empty());
    let timeout_millis = input.timeout_millis.unwrap_or(10_000);
    let max_response_bytes = input.max_response_bytes.unwrap_or(2 * 1024 * 1024);
    validate_limits(input.timeout_millis, input.max_response_bytes)?;
    let device_id = input.device_id.as_deref().unwrap_or(DEFAULT_DEVICE_ID);
    let device_name = input.device_name.as_deref().unwrap_or(DEFAULT_DEVICE_NAME);
    validate_identity(Some(device_id), "device ID")?;
    validate_identity(Some(device_name), "device name")?;
    let fields = &input.fields;
    let key_events = |events: Vec<VizioRemoteEvent>| {
        json!({"KEYLIST": events.into_iter().map(|event| json!({
            "CODESET": event.code_set,
            "CODE": event.code,
            "ACTION": action_name(&event.action),
        })).collect::<Vec<_>>()})
    };
    let (method, path, body, auth_required) = match operation {
        "beginPair" => (
            VizioHttpMethod::Put,
            "/pairing/start".into(),
            Some(json!({
                "DEVICE_ID": device_id,
                "DEVICE_NAME": device_name,
            })),
            false,
        ),
        "finishPair" => {
            let challenge_type = required_u64(fields, "challengeType")?;
            let token = required_u64(fields, "token")?;
            let pin = required_string(fields, "pin")?;
            if challenge_type > u16::MAX.into()
                || token > u32::MAX.into()
                || pin.len() > 32
                || pin.is_empty()
                || pin.chars().any(char::is_control)
            {
                return Err(failure(
                    VizioFailureKind::InvalidParameter,
                    "Invalid pairing PIN",
                    false,
                ));
            }
            (
                VizioHttpMethod::Put,
                "/pairing/pair".into(),
                Some(json!({
                    "DEVICE_ID": device_id,
                    "CHALLENGE_TYPE": challenge_type,
                    "RESPONSE_VALUE": pin,
                    "PAIRING_REQ_TOKEN": token,
                })),
                false,
            )
        }
        "cancelPair" => (
            VizioHttpMethod::Put,
            "/pairing/cancel".into(),
            Some(json!({
                "DEVICE_ID": device_id,
                "DEVICE_NAME": device_name,
                "CHALLENGE_TYPE": 1,
                "RESPONSE_VALUE": "1111",
                "PAIRING_REQ_TOKEN": 0,
            })),
            false,
        ),
        "ping" | "deviceInfo" => (
            VizioHttpMethod::Get,
            "/state/device/deviceinfo".into(),
            None,
            false,
        ),
        "pingAuth" | "powerState" => (
            VizioHttpMethod::Get,
            "/state/device/power_mode".into(),
            None,
            true,
        ),
        "remote" => {
            let events = serde_json::from_value::<Vec<VizioRemoteEvent>>(
                fields.get("events").cloned().unwrap_or(Value::Null),
            )
            .map_err(|_| {
                failure(
                    VizioFailureKind::InvalidInput,
                    "Invalid remote event list",
                    false,
                )
            })?;
            if events.is_empty() {
                return Err(failure(
                    VizioFailureKind::InvalidInput,
                    "Remote event list is empty",
                    false,
                ));
            }
            if events.len() > 256 {
                return Err(failure(
                    VizioFailureKind::InvalidParameter,
                    "Remote event list is too large",
                    false,
                ));
            }
            (
                VizioHttpMethod::Put,
                "/key_command/".into(),
                Some(key_events(events)),
                true,
            )
        }
        "key" => {
            let key = serde_json::from_value::<VizioRemoteKey>(
                fields.get("key").cloned().unwrap_or(Value::Null),
            )
            .map_err(|_| {
                failure(
                    VizioFailureKind::InvalidInput,
                    "Unknown SmartCast remote key",
                    false,
                )
            })?;
            let action = fields
                .get("action")
                .cloned()
                .map(serde_json::from_value::<VizioRemoteAction>)
                .transpose()
                .map_err(|_| {
                    failure(
                        VizioFailureKind::InvalidInput,
                        "Invalid remote action",
                        false,
                    )
                })?
                .unwrap_or(VizioRemoteAction::Keypress);
            (
                VizioHttpMethod::Put,
                "/key_command/".into(),
                Some(key_events(vec![key.event(action)])),
                true,
            )
        }
        "text" => {
            let text = required_string(fields, "text")?;
            if !text.is_ascii() || text.len() > 512 {
                return Err(failure(
                    VizioFailureKind::InvalidParameter,
                    "SmartCast text entry supports at most 512 ASCII characters",
                    false,
                ));
            }
            let events = text
                .bytes()
                .map(|code| VizioRemoteEvent {
                    code_set: 0,
                    code: u16::from(code),
                    action: VizioRemoteAction::Keypress,
                })
                .collect();
            (
                VizioHttpMethod::Put,
                "/key_command/".into(),
                Some(key_events(events)),
                true,
            )
        }
        "powerOn" => key_request(VizioRemoteKey::PowOn, key_events),
        "powerOff" => key_request(VizioRemoteKey::PowOff, key_events),
        "powerToggle" => key_request(VizioRemoteKey::PowToggle, key_events),
        "volumeUp" => repeated_key(VizioRemoteKey::VolUp, fields, key_events)?,
        "volumeDown" => repeated_key(VizioRemoteKey::VolDown, fields, key_events)?,
        "mute" => key_request(VizioRemoteKey::MuteOn, key_events),
        "unmute" => key_request(VizioRemoteKey::MuteOff, key_events),
        "muteToggle" => key_request(VizioRemoteKey::MuteToggle, key_events),
        "nextInput" => key_request(VizioRemoteKey::InputNext, key_events),
        "channelUp" => key_request(VizioRemoteKey::ChUp, key_events),
        "channelDown" => key_request(VizioRemoteKey::ChDown, key_events),
        "previousChannel" => key_request(VizioRemoteKey::ChPrev, key_events),
        "getSettings" => {
            let category = setting_path(required_string(fields, "category")?)?;
            (
                VizioHttpMethod::Get,
                format!("{SETTINGS_ROOT}/{category}"),
                None,
                true,
            )
        }
        "getSetting" | "getVolume" | "isMuted" => {
            let (category, name) = if operation == "getVolume" {
                ("audio", "volume")
            } else if operation == "isMuted" {
                ("audio", "mute")
            } else {
                (
                    required_string(fields, "category")?,
                    required_string(fields, "name")?,
                )
            };
            (
                VizioHttpMethod::Get,
                format!(
                    "{SETTINGS_ROOT}/{}/{}",
                    setting_path(category)?,
                    setting_path(name)?
                ),
                None,
                true,
            )
        }
        "setSetting" => {
            let category = setting_path(required_string(fields, "category")?)?;
            let name = setting_path(required_string(fields, "name")?)?;
            let value = fields.get("value").cloned().ok_or_else(|| {
                failure(
                    VizioFailureKind::InvalidInput,
                    "Setting value is required",
                    false,
                )
            })?;
            validate_setting_value(&value, fields)?;
            let hash = required_i64(fields, "hashValue")?;
            (
                VizioHttpMethod::Put,
                format!("{SETTINGS_ROOT}/{category}/{name}"),
                Some(json!({
                    "REQUEST": "MODIFY", "VALUE": value, "HASHVAL": hash,
                })),
                true,
            )
        }
        "triggerSetting" | "blankScreen" => {
            let (category, name) = if operation == "blankScreen" {
                ("system/timers", "blank_screen")
            } else {
                (
                    required_string(fields, "category")?,
                    required_string(fields, "name")?,
                )
            };
            let hash = required_i64(fields, "hashValue")?;
            (
                VizioHttpMethod::Put,
                format!(
                    "{SETTINGS_ROOT}/{}/{}",
                    setting_path(category)?,
                    setting_path(name)?
                ),
                Some(json!({
                    "REQUEST": "ACTION", "HASHVAL": hash,
                })),
                true,
            )
        }
        "setVolume" => {
            let level = required_i64(fields, "level")?;
            if !(0..=100).contains(&level) {
                return Err(failure(
                    VizioFailureKind::InvalidParameter,
                    "TV volume must be between 0 and 100",
                    false,
                ));
            }
            (
                VizioHttpMethod::Put,
                "/audio/volume/level".into(),
                Some(json!({"VALUE": level})),
                true,
            )
        }
        "getInputs" => (
            VizioHttpMethod::Get,
            format!("{SETTINGS_ROOT}/devices/name_input"),
            None,
            true,
        ),
        "getCurrentInput" => (
            VizioHttpMethod::Get,
            format!("{SETTINGS_ROOT}/devices/current_input"),
            None,
            true,
        ),
        "setInput" => {
            let cname = required_string(fields, "cname")?;
            let hash = required_i64(fields, "hashValue")?;
            if cname.is_empty() || cname.len() > 128 {
                return Err(failure(
                    VizioFailureKind::InvalidParameter,
                    "Invalid TV input CNAME",
                    false,
                ));
            }
            (
                VizioHttpMethod::Put,
                format!("{SETTINGS_ROOT}/devices/current_input"),
                Some(json!({
                    "REQUEST": "MODIFY", "VALUE": cname, "HASHVAL": hash,
                })),
                true,
            )
        }
        "launchApp" => {
            let app_id = required_string(fields, "appId")?;
            let name_space = required_u64(fields, "nameSpace")?;
            if app_id.is_empty() || app_id.len() > 128 || name_space > u16::MAX.into() {
                return Err(failure(
                    VizioFailureKind::InvalidParameter,
                    "Invalid SmartCast app configuration",
                    false,
                ));
            }
            let mut value = json!({"APP_ID":app_id,"NAME_SPACE":name_space});
            if let Some(message) = fields.get("message").and_then(Value::as_str) {
                if message.len() > 4096 || message.chars().any(char::is_control) {
                    return Err(failure(
                        VizioFailureKind::InvalidParameter,
                        "Invalid SmartCast app message",
                        false,
                    ));
                }
                value["MESSAGE"] = json!(message);
            }
            (
                VizioHttpMethod::Put,
                "/app/launch".into(),
                Some(json!({"VALUE":value})),
                true,
            )
        }
        "launchConjure" => {
            let url = validate_conjure_url(required_string(fields, "url")?)?;
            let mut value = json!({"APP_ID":"17","NAME_SPACE":4,"MESSAGE":url});
            if fields
                .get("debug")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                value["DEBUG"] = json!(1);
            }
            (
                VizioHttpMethod::Put,
                "/app/launch".into(),
                Some(json!({"VALUE":value})),
                true,
            )
        }
        "currentApp" => (VizioHttpMethod::Get, "/app/current".into(), None, true),
        "stateExtended" => (VizioHttpMethod::Get, "/state_extended".into(), None, true),
        "systemVersions" => (VizioHttpMethod::Get, "/system/versions".into(), None, true),
        "pinDefault" => (
            VizioHttpMethod::Get,
            "/pin/is_pin_default".into(),
            None,
            true,
        ),
        "identityPrimary" | "identityLegacy" => {
            let item = setting_path(required_string(fields, "item")?)?;
            let section = setting_path(required_string(fields, "section")?)?;
            let root = if operation == "identityPrimary" {
                "admin_and_privacy/system_information"
            } else {
                "system/system_information"
            };
            (
                VizioHttpMethod::Get,
                format!("{SETTINGS_ROOT}/{root}/{section}/{item}"),
                None,
                true,
            )
        }
        _ => {
            return Err(failure(
                VizioFailureKind::InvalidInput,
                "Unknown SmartCast operation",
                false,
            ));
        }
    };
    build_request(
        base,
        auth,
        method,
        &path,
        body,
        auth_required,
        RequestLimits {
            timeout_millis,
            max_response_bytes,
        },
    )
}

fn key_request(
    key: VizioRemoteKey,
    key_events: impl FnOnce(Vec<VizioRemoteEvent>) -> Value,
) -> (VizioHttpMethod, String, Option<Value>, bool) {
    (
        VizioHttpMethod::Put,
        "/key_command/".into(),
        Some(key_events(vec![key.event(VizioRemoteAction::Keypress)])),
        true,
    )
}

fn repeated_key(
    key: VizioRemoteKey,
    fields: &Map<String, Value>,
    key_events: impl FnOnce(Vec<VizioRemoteEvent>) -> Value,
) -> Result<(VizioHttpMethod, String, Option<Value>, bool), VizioFailure> {
    let steps = fields
        .get("steps")
        .and_then(Value::as_u64)
        .unwrap_or(1)
        .max(1);
    if steps > 100 {
        return Err(failure(
            VizioFailureKind::InvalidParameter,
            "Remote repeat is too large",
            false,
        ));
    }
    Ok(key_request_events(
        (0..steps)
            .map(|_| key.event(VizioRemoteAction::Keypress))
            .collect(),
        key_events,
    ))
}

fn key_request_events(
    events: Vec<VizioRemoteEvent>,
    key_events: impl FnOnce(Vec<VizioRemoteEvent>) -> Value,
) -> (VizioHttpMethod, String, Option<Value>, bool) {
    (
        VizioHttpMethod::Put,
        "/key_command/".into(),
        Some(key_events(events)),
        true,
    )
}

fn build_request(
    base: String,
    auth: Option<&str>,
    method: VizioHttpMethod,
    path: &str,
    body: Option<Value>,
    auth_required: bool,
    limits: RequestLimits,
) -> Result<VizioRequest, VizioFailure> {
    if auth_required && auth.is_none() {
        return Err(failure(
            VizioFailureKind::Authentication,
            "This SmartCast endpoint requires pairing",
            false,
        ));
    }
    let mut headers = BTreeMap::from([
        ("Accept".into(), "application/json".into()),
        ("Content-Type".into(), "application/json".into()),
    ]);
    if auth_required {
        headers.insert("AUTH".into(), auth.unwrap_or_default().into());
    }
    let body = body.map(value_object).transpose()?;
    Ok(VizioRequest {
        method,
        url: format!(
            "{base}{}",
            if path.starts_with('/') {
                path.into()
            } else {
                format!("/{path}")
            }
        ),
        headers,
        body,
        timeout_millis: limits.timeout_millis,
        max_response_bytes: limits.max_response_bytes,
    })
}

fn normalize_host(host: &str) -> Result<String, VizioFailure> {
    let host = host.trim().trim_end_matches('/');
    if host.is_empty() {
        return Err(failure(
            VizioFailureKind::InvalidConfig,
            "TV host is required",
            false,
        ));
    }
    if host.to_ascii_lowercase().starts_with("http://") {
        return Err(failure(
            VizioFailureKind::InvalidConfig,
            "SmartCast TV connections require HTTPS",
            false,
        ));
    }
    let without_scheme = host
        .strip_prefix("https://")
        .or_else(|| host.strip_prefix("HTTPS://"))
        .unwrap_or(host);
    let authority = without_scheme.split('/').next().unwrap_or_default();
    let has_explicit_port = if authority.starts_with('[') {
        authority
            .find(']')
            .is_some_and(|end| authority.as_bytes().get(end + 1) == Some(&b':'))
    } else {
        authority.rsplit_once(':').is_some_and(|(_, port)| {
            !port.is_empty() && port.bytes().all(|byte| byte.is_ascii_digit())
        })
    };
    let mut url = url::Url::parse(&format!("https://{without_scheme}"))
        .map_err(|_| failure(VizioFailureKind::InvalidConfig, "Invalid TV host", false))?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
        || url.host_str().is_none()
    {
        return Err(failure(
            VizioFailureKind::InvalidConfig,
            "Invalid TV host",
            false,
        ));
    }
    if !has_explicit_port {
        url.set_port(Some(7345))
            .map_err(|_| failure(VizioFailureKind::InvalidConfig, "Invalid TV host", false))?;
    }
    Ok(url.origin().ascii_serialization())
}

fn validate_conjure_url(input: &str) -> Result<String, VizioFailure> {
    let url = url::Url::parse(input).map_err(|_| {
        failure(
            VizioFailureKind::InvalidParameter,
            "Conjure apps require an HTTP or HTTPS URL reachable by the TV",
            false,
        )
    })?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(failure(
            VizioFailureKind::InvalidParameter,
            "Conjure apps require an HTTP or HTTPS URL reachable by the TV",
            false,
        ));
    }
    Ok(url.to_string())
}

fn parse_protocol_response(
    status: u16,
    body: &str,
    allow_statusless: bool,
) -> Result<VizioProtocolResponse, VizioFailure> {
    if status == 401 || status == 403 {
        return Err(failure(
            VizioFailureKind::Authentication,
            "TV rejected the pairing token",
            false,
        ));
    }
    if !(200..=299).contains(&status) {
        return Err(failure(
            VizioFailureKind::HttpStatus,
            &format!("Vizio request failed with HTTP {status}"),
            status == 408 || status == 429 || status >= 500,
        ));
    }
    if body.len() > 8 * 1024 * 1024 {
        return Err(failure(
            VizioFailureKind::InvalidResponse,
            "TV response exceeded the size limit",
            false,
        ));
    }
    let raw_value = serde_json::from_str::<Value>(body).map_err(|_| {
        failure(
            VizioFailureKind::InvalidResponse,
            "TV returned invalid JSON",
            false,
        )
    })?;
    let raw = raw_value.as_object().ok_or_else(|| {
        failure(
            VizioFailureKind::InvalidResponse,
            "TV returned invalid JSON",
            false,
        )
    })?;
    let status_record = ci_field(raw, "STATUS").and_then(Value::as_object);
    if status_record.is_none() && allow_statusless {
        return Ok(VizioProtocolResponse {
            status: "SUCCESS".into(),
            detail: "Success".into(),
            raw: value_object(raw_value)?,
        });
    }
    let status_record = status_record.ok_or_else(|| {
        failure(
            VizioFailureKind::InvalidResponse,
            "TV response is missing STATUS",
            false,
        )
    })?;
    let reported_result = ci_field(status_record, "RESULT")
        .and_then(Value::as_str)
        .unwrap_or("UNKNOWN")
        .to_ascii_uppercase();
    let result = match reported_result.as_str() {
        "SUCCESS"
        | "REQUIRES_PAIRING"
        | "PAIRING_DENIED"
        | "CHALLENGE_INCORRECT"
        | "INVALID_PARAMETER"
        | "HASHVAL_ERROR"
        | "VALUE_OUT_OF_RANGE"
        | "URI_NOT_FOUND"
        | "BLOCKED"
        | "MAX_CHALLENGES_EXCEEDED" => reported_result,
        _ => "UNKNOWN".to_owned(),
    };
    let detail = ci_field(status_record, "DETAIL")
        .and_then(Value::as_str)
        .unwrap_or(&result)
        .to_owned();
    let error_kind = match result.as_str() {
        "SUCCESS" => {
            return Ok(VizioProtocolResponse {
                status: result,
                detail,
                raw: value_object(raw_value)?,
            });
        }
        "REQUIRES_PAIRING" | "PAIRING_DENIED" | "CHALLENGE_INCORRECT" => {
            VizioFailureKind::Authentication
        }
        "INVALID_PARAMETER" | "HASHVAL_ERROR" | "VALUE_OUT_OF_RANGE" => {
            VizioFailureKind::InvalidParameter
        }
        "URI_NOT_FOUND" => VizioFailureKind::EndpointNotFound,
        "BLOCKED" | "MAX_CHALLENGES_EXCEEDED" => VizioFailureKind::Busy,
        _ => VizioFailureKind::InvalidResponse,
    };
    let retryable = error_kind == VizioFailureKind::Busy;
    let message = match error_kind {
        VizioFailureKind::Authentication => "TV rejected SmartCast authentication",
        VizioFailureKind::InvalidParameter => "TV rejected a SmartCast parameter",
        VizioFailureKind::EndpointNotFound => "TV does not support this SmartCast endpoint",
        VizioFailureKind::Busy => "TV is busy with another SmartCast request",
        _ => "TV rejected the SmartCast request",
    };
    let mut error = failure(error_kind, message, retryable);
    error.protocol_status = Some(result);
    Err(error)
}

fn validate_setting_value(value: &Value, fields: &Map<String, Value>) -> Result<(), VizioFailure> {
    if let Some(number) = value.as_f64() {
        if fields
            .get("minimum")
            .and_then(Value::as_f64)
            .is_some_and(|minimum| number < minimum)
        {
            return Err(failure(
                VizioFailureKind::InvalidParameter,
                "Setting is below its minimum",
                false,
            ));
        }
        if fields
            .get("maximum")
            .and_then(Value::as_f64)
            .is_some_and(|maximum| number > maximum)
        {
            return Err(failure(
                VizioFailureKind::InvalidParameter,
                "Setting is above its maximum",
                false,
            ));
        }
    }
    if let Some(text) = value.as_str()
        && let Some(options) = fields.get("options").and_then(Value::as_array)
        && !options.is_empty()
        && !options
            .iter()
            .filter_map(Value::as_str)
            .any(|option| option.eq_ignore_ascii_case(text))
    {
        return Err(failure(
            VizioFailureKind::InvalidParameter,
            "Setting value is unsupported",
            false,
        ));
    }
    Ok(())
}

fn setting_path(value: &str) -> Result<&str, VizioFailure> {
    if value.is_empty()
        || value.len() > 256
        || value.starts_with('/')
        || value.ends_with('/')
        || value.split('/').any(|part| {
            part.is_empty()
                || !part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        })
    {
        return Err(failure(
            VizioFailureKind::InvalidParameter,
            "Invalid SmartCast setting path",
            false,
        ));
    }
    Ok(value)
}

fn action_name(action: &VizioRemoteAction) -> &'static str {
    match action {
        VizioRemoteAction::Keypress => "KEYPRESS",
        VizioRemoteAction::Keydown => "KEYDOWN",
        VizioRemoteAction::Keyup => "KEYUP",
    }
}

fn required_string<'a>(
    fields: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a str, VizioFailure> {
    fields.get(name).and_then(Value::as_str).ok_or_else(|| {
        failure(
            VizioFailureKind::InvalidInput,
            "SmartCast command is missing required data",
            false,
        )
    })
}
fn required_u64(fields: &Map<String, Value>, name: &str) -> Result<u64, VizioFailure> {
    fields.get(name).and_then(Value::as_u64).ok_or_else(|| {
        failure(
            VizioFailureKind::InvalidInput,
            "SmartCast command is missing required data",
            false,
        )
    })
}
fn required_i64(fields: &Map<String, Value>, name: &str) -> Result<i64, VizioFailure> {
    fields.get(name).and_then(Value::as_i64).ok_or_else(|| {
        failure(
            VizioFailureKind::InvalidInput,
            "SmartCast command is missing required data",
            false,
        )
    })
}
fn ci_field<'a>(value: &'a Map<String, Value>, name: &str) -> Option<&'a Value> {
    value
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value)
}
fn record<'a>(value: &'a JsonObject, name: &str) -> Option<&'a JsonObject> {
    value
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .and_then(|(_, value)| match value {
            JsonValue::Object(value) => Some(value),
            _ => None,
        })
}
fn string<'a>(value: &'a JsonObject, name: &str) -> Option<&'a str> {
    value
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .and_then(|(_, value)| match value {
            JsonValue::String(value) => Some(value.as_str()),
            _ => None,
        })
}
fn number(value: &JsonObject, name: &str) -> Option<f64> {
    value
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .and_then(|(_, value)| match value {
            JsonValue::Number(value) => Some(*value),
            JsonValue::String(value) => value.parse().ok(),
            _ => None,
        })
}
fn response_item(raw: &JsonObject) -> Option<&JsonObject> {
    record(raw, "ITEM")
}
fn response_items(raw: &JsonObject) -> Vec<&JsonObject> {
    raw.iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("ITEMS"))
        .and_then(|(_, value)| match value {
            JsonValue::Array(values) => Some(values),
            _ => None,
        })
        .into_iter()
        .flatten()
        .filter_map(|value| match value {
            JsonValue::Object(value) => Some(value),
            _ => None,
        })
        .collect()
}
fn number_to_u16(value: f64) -> Option<u16> {
    (value.fract() == 0.0 && value >= 0.0 && value <= u16::MAX as f64).then_some(value as u16)
}
fn number_to_u32(value: f64) -> Option<u32> {
    (value.fract() == 0.0 && value >= 0.0 && value <= u32::MAX as f64).then_some(value as u32)
}
fn number_to_i64(value: f64) -> Option<i64> {
    (value.is_finite()
        && value.fract() == 0.0
        && value >= i64::MIN as f64
        && value <= i64::MAX as f64)
        .then_some(value as i64)
}
fn value_object(value: Value) -> Result<JsonObject, VizioFailure> {
    serde_json::from_value(value).map_err(|_| {
        failure(
            VizioFailureKind::InvalidResponse,
            "Invalid SmartCast JSON object",
            false,
        )
    })
}
fn failure(kind: VizioFailureKind, message: &str, retryable: bool) -> VizioFailure {
    VizioFailure {
        kind,
        message: message.into(),
        retryable,
        protocol_status: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(extra: Value) -> String {
        let mut base = json!({"host":"https://192.0.2.4/","authToken":"secret"});
        base.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        base.to_string()
    }

    #[test]
    fn upstream_remote_and_conjure_contract_is_preserved() {
        let VizioRequestResult::Ok(home) = plan_request("key", &input(json!({"key":"HOME"})))
        else {
            panic!()
        };
        assert_eq!(home.url, "https://192.0.2.4:7345/key_command/");
        assert_eq!(
            home.body.unwrap()["KEYLIST"],
            json_value(json!([{"CODESET":4,"CODE":15,"ACTION":"KEYPRESS"}]))
        );
        let VizioRequestResult::Ok(conjure) = plan_request(
            "launchConjure",
            &input(json!({"url":"https://example.invalid/tv/"})),
        ) else {
            panic!()
        };
        assert_eq!(
            conjure.body.unwrap()["VALUE"],
            json_value(
                json!({"APP_ID":"17","NAME_SPACE":4,"MESSAGE":"https://example.invalid/tv/"})
            )
        );
    }

    #[test]
    fn pairing_and_protocol_status_are_case_insensitive() {
        let response = parse_protocol_response(
            200,
            r#"{"status":{"result":"SUCCESS"},"item":{"CHALLENGE_TYPE":1,"PAIRING_REQ_TOKEN":42}}"#,
            false,
        )
        .unwrap();
        let challenge = pairing_challenge(&response).unwrap();
        assert_eq!(challenge.challenge_type, 1);
        assert_eq!(challenge.token, 42);
        let VizioResponseResult::Err(error) = parse_response(
            200,
            r#"{"STATUS":{"RESULT":"HASHVAL_ERROR","DETAIL":"stale"}}"#,
            false,
        ) else {
            panic!()
        };
        assert_eq!(error.kind, VizioFailureKind::InvalidParameter);
    }

    #[test]
    fn text_limits_and_input_freshness_payload_are_enforced() {
        assert!(matches!(
            plan_request("text", &input(json!({"text":"Air 📺"}))),
            VizioRequestResult::Err(_)
        ));
        let VizioRequestResult::Ok(request) =
            plan_request("setInput", &input(json!({"cname":"hdmi2","hashValue":12})))
        else {
            panic!()
        };
        let body = request.body.unwrap();
        assert_eq!(body["VALUE"], JsonValue::String("hdmi2".into()));
        assert_eq!(body["HASHVAL"], JsonValue::Number(12.0));
    }

    #[test]
    fn discovery_is_bounded_and_platform_limits_are_explicit() {
        let candidates = discovery_candidates("192.168.1").unwrap();
        assert_eq!(candidates.len(), 508);
        assert_eq!(candidates[0].host, "192.168.1.1:7345");
        assert!(matches!(
            platform_support("android").transport,
            VizioTransportSupport::Native
        ));
        assert!(matches!(
            platform_support("tizen").transport,
            VizioTransportSupport::Unavailable
        ));
        assert!(matches!(
            platform_support("android-tv").transport,
            VizioTransportSupport::Unavailable
        ));
    }

    #[test]
    fn controller_adopts_pairing_token_before_next_command() {
        let mut controller = VizioController::new(
            r#"{"host":"192.0.2.8","deviceId":"stable-device","deviceName":"VIPTV mobile"}"#,
        )
        .unwrap();
        let start = controller.start(
            "finishPair",
            r#"{"challengeType":1,"token":42,"pin":"1234"}"#,
        );
        let id = start.request_id.unwrap();
        let done = controller.resolve(
            id,
            200,
            r#"{"STATUS":{"RESULT":"SUCCESS"},"ITEM":{"AUTH_TOKEN":"new-secret"}}"#,
        );
        assert_eq!(done.kind, VizioControllerOutputKind::Complete);
        assert!(done.credential_changed);
        assert_eq!(controller.auth_token().as_deref(), Some("new-secret"));

        let command = controller.start("powerOn", "{}");
        assert_eq!(
            command
                .request
                .unwrap()
                .headers
                .get("AUTH")
                .map(String::as_str),
            Some("new-secret")
        );
    }

    #[test]
    fn controller_uses_target_cname_and_a_fresh_current_input_hash() {
        let mut controller = paired_controller();
        let first = controller.start("setInput", r#"{"input":"Game console"}"#);
        assert!(
            first
                .request
                .as_ref()
                .unwrap()
                .url
                .ends_with("/devices/current_input")
        );
        let list = controller.resolve(
            first.request_id.unwrap(),
            200,
            &success(json!({"ITEM":{"CNAME":"current_input","VALUE":"hdmi1","HASHVAL":7}})),
        );
        assert!(
            list.request
                .as_ref()
                .unwrap()
                .url
                .ends_with("/devices/name_input")
        );
        let fresh = controller.resolve(
            list.request_id.unwrap(),
            200,
            &success(json!({"ITEMS":[
                {"CNAME":"hdmi1","NAME":"HDMI-1","VALUE":{"NAME":"Cable"}},
                {"CNAME":"hdmi2","NAME":"HDMI-2","VALUE":{"NAME":"Game console"}}
            ]})),
        );
        assert!(
            fresh
                .request
                .as_ref()
                .unwrap()
                .url
                .ends_with("/devices/current_input")
        );
        let write = controller.resolve(
            fresh.request_id.unwrap(),
            200,
            &success(json!({"ITEM":{"CNAME":"current_input","VALUE":"hdmi1","HASHVAL":99}})),
        );
        let body = write.request.unwrap().body.unwrap();
        assert_eq!(body["VALUE"], JsonValue::String("hdmi2".into()));
        assert_eq!(body["HASHVAL"], JsonValue::Number(99.0));
    }

    #[test]
    fn controller_retries_a_stale_setting_hash_once() {
        let mut controller = paired_controller();
        let read = controller.start(
            "setSetting",
            r#"{"category":"audio","name":"volume","value":44}"#,
        );
        let first_write = controller.resolve(
            read.request_id.unwrap(),
            200,
            &setting_response("volume", 10, 0, 100),
        );
        assert_eq!(
            first_write.request.as_ref().unwrap().body.as_ref().unwrap()["HASHVAL"],
            JsonValue::Number(10.0)
        );
        let reread = controller.resolve(
            first_write.request_id.unwrap(),
            200,
            r#"{"STATUS":{"RESULT":"HASHVAL_ERROR"}}"#,
        );
        assert!(matches!(
            reread.request.as_ref().unwrap().method,
            VizioHttpMethod::Get
        ));
        let second_write = controller.resolve(
            reread.request_id.unwrap(),
            200,
            &setting_response("volume", 11, 0, 100),
        );
        assert_eq!(
            second_write
                .request
                .as_ref()
                .unwrap()
                .body
                .as_ref()
                .unwrap()["HASHVAL"],
            JsonValue::Number(11.0)
        );
        let failed = controller.resolve(
            second_write.request_id.unwrap(),
            200,
            r#"{"STATUS":{"RESULT":"HASHVAL_ERROR"}}"#,
        );
        assert_eq!(failed.kind, VizioControllerOutputKind::Error);
        assert_eq!(
            failed.error.unwrap().protocol_status.as_deref(),
            Some("HASHVAL_ERROR")
        );
    }

    #[test]
    fn controller_serializes_commands_and_ignores_stale_responses() {
        let mut controller = paired_controller();
        let pending = controller.start("powerOn", "{}");
        assert_eq!(
            controller.start("powerOff", "{}").error.unwrap().kind,
            VizioFailureKind::Busy
        );
        assert_eq!(
            controller
                .resolve(pending.request_id.unwrap() + 1, 200, "{}")
                .kind,
            VizioControllerOutputKind::Error
        );
        assert!(controller.pending.is_some());
        controller.cancel();
        assert!(controller.pending.is_none());
    }

    #[test]
    fn remote_table_and_status_redaction_match_the_upstream_contract() {
        let expected = [
            (VizioRemoteKey::SeekFwd, 2, 0),
            (VizioRemoteKey::SeekBack, 2, 1),
            (VizioRemoteKey::Pause, 2, 2),
            (VizioRemoteKey::Play, 2, 3),
            (VizioRemoteKey::Down, 3, 0),
            (VizioRemoteKey::Left, 3, 1),
            (VizioRemoteKey::Ok, 3, 2),
            (VizioRemoteKey::Right, 3, 7),
            (VizioRemoteKey::Up, 3, 8),
            (VizioRemoteKey::Back, 4, 0),
            (VizioRemoteKey::Smartcast, 4, 3),
            (VizioRemoteKey::CcToggle, 4, 4),
            (VizioRemoteKey::Info, 4, 6),
            (VizioRemoteKey::Menu, 4, 8),
            (VizioRemoteKey::Home, 4, 15),
            (VizioRemoteKey::VolDown, 5, 0),
            (VizioRemoteKey::VolUp, 5, 1),
            (VizioRemoteKey::MuteOff, 5, 2),
            (VizioRemoteKey::MuteOn, 5, 3),
            (VizioRemoteKey::MuteToggle, 5, 4),
            (VizioRemoteKey::PicMode, 6, 0),
            (VizioRemoteKey::PicSize, 6, 2),
            (VizioRemoteKey::InputNext, 7, 1),
            (VizioRemoteKey::ChDown, 8, 0),
            (VizioRemoteKey::ChUp, 8, 1),
            (VizioRemoteKey::ChPrev, 8, 2),
            (VizioRemoteKey::Exit, 9, 0),
            (VizioRemoteKey::PowOff, 11, 0),
            (VizioRemoteKey::PowOn, 11, 1),
            (VizioRemoteKey::PowToggle, 11, 2),
        ];
        for (key, code_set, code) in expected {
            let event = key.event(VizioRemoteAction::Keypress);
            assert_eq!((event.code_set, event.code), (code_set, code));
        }
        let VizioResponseResult::Err(error) = parse_response(
            200,
            r#"{"STATUS":{"RESULT":"token=should-not-escape"}}"#,
            false,
        ) else {
            panic!()
        };
        assert_eq!(error.protocol_status.as_deref(), Some("UNKNOWN"));
    }

    #[test]
    fn explicit_https_port_is_preserved() {
        let VizioRequestResult::Ok(request) =
            plan_request("ping", r#"{"host":"https://example.test:443"}"#)
        else {
            panic!()
        };
        assert_eq!(request.url, "https://example.test/state/device/deviceinfo");
    }

    fn paired_controller() -> VizioController {
        VizioController::new(r#"{"host":"192.0.2.8","authToken":"secret"}"#).unwrap()
    }

    fn success(payload: Value) -> String {
        let mut object = payload.as_object().unwrap().clone();
        object.insert("STATUS".into(), json!({"RESULT":"SUCCESS"}));
        Value::Object(object).to_string()
    }

    fn setting_response(name: &str, hash: i64, minimum: i64, maximum: i64) -> String {
        success(json!({"ITEM":{
            "CNAME": name,
            "VALUE": 20,
            "HASHVAL": hash,
            "MINIMUM": minimum,
            "MAXIMUM": maximum
        }}))
    }

    fn json_value(value: Value) -> JsonValue {
        serde_json::from_value(value).unwrap()
    }
}
