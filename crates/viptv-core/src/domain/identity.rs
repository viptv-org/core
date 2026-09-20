use super::*;

pub fn clean(v: &Value) -> Value {
    match v {
        Value::Array(items) => Value::Array(items.iter().map(clean).collect()),
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .filter(|(k, _)| {
                    let k = k.to_lowercase().replace(['_', '-'], "");
                    ![
                        "url",
                        "uri",
                        "link",
                        "header",
                        "authorization",
                        "accesstoken",
                        "refreshtoken",
                        "devicecode",
                        "devicetoken",
                        "cookie",
                        "password",
                        "credential",
                        "proxy",
                        "referer",
                        "origin",
                    ]
                    .iter()
                    .any(|s| k.contains(s))
                })
                .map(|(k, v)| (k.clone(), clean(v)))
                .collect(),
        ),
        _ => v.clone(),
    }
}
pub fn profile(v: &Value) -> Result<Value> {
    obj(v)?;
    let mut out = json!({"id":id(v,"id")?,"name":string(v,"name")?,"raw":clean(v)});
    for (a, b) in [
        ("primary", "primary"),
        ("is_primary", "primary"),
        ("avatar_style", "avatarStyle"),
        ("avatar_choice", "avatarChoice"),
    ] {
        copy(v, &mut out, a, b);
    }
    if let Some(avatar) = v["avatar"].as_str().or_else(|| v["avatar_url"].as_str()) {
        out["avatar"] = json!(avatar);
    }
    if let Some(kid) = v["kids"]
        .as_bool()
        .or_else(|| v["kid"].as_bool())
        .or_else(|| v["is_kids"].as_bool())
    {
        out["kid"] = json!(kid);
    }
    if let Some(complete) = v["setup_complete"].as_bool() {
        out["setupComplete"] = json!(complete);
    }
    Ok(out)
}
pub fn identity(v: &Value) -> Result<Value> {
    let a = &v["account"];
    Ok(
        json!({"account":{"id":id(a,"id")?,"username":string(a,"username")?,"name":string(a,"name")?,"role":string(a,"role")?},
        "profiles":array(v,"profiles")?.iter().map(profile).collect::<Result<Vec<_>>>()?,
        "profileId":id(v,"profile_id").ok(),"restricted":boolean(v,"restricted")?,"profileSetupRequired":boolean(v,"profile_setup_required")?}),
    )
}
pub fn tokens(v: &Value) -> Result<Value> {
    Ok(
        json!({"sessionId":id(v,"session_id")?,"accountId":id(v,"account_id")?,"profileId":id(v,"profile_id").ok(),"accessToken":string(v,"access_token")?,"refreshToken":string(v,"refresh_token")?,"expiresIn":number(v,"expires_in")?}),
    )
}
