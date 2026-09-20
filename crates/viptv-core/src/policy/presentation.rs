use super::*;

pub(super) fn presentation(v: &Value) -> Result {
    Ok({
        let m = if v["item"].is_object() { &v["item"] } else { v };
        let episode = num(m, "episode");
        let season = num(m, "season");
        let label = if episode > 0.0 {
            format!(
                "S{season:.0} E{episode:.0}{}",
                if text(m, "episodeTitle").is_empty() {
                    String::new()
                } else {
                    format!(" · {}", text(m, "episodeTitle"))
                }
            )
        } else {
            String::new()
        };
        let live = text(m, "type") == "live";
        let next = text(m, "queueStatus") == "next";
        let resume = !live && num(m, "position") > 0.0;
        let action = if live {
            "play"
        } else if next {
            "next"
        } else if resume {
            "resume"
        } else if text(m, "type") == "series" && episode <= 0.0 {
            "episodes"
        } else {
            "sources"
        };
        json!({"heroImage":image(m,"background"),"posterImage":image(m,"poster"),"episodeImage":image(m,"thumbnail"),"titleLogo":image(m,"titleLogo"),"title":m["name"],"episodeLabel":label,"progress":if num(m,"duration")>0.0{(num(m,"position")/num(m,"duration")).clamp(0.0,1.0)}else{0.0},"primaryAction":action,"primaryActionLabel":match action{"next"=>"Play next episode","resume"=>"Resume","play"=>"Watch live","episodes"=>"Episodes",_=>"Play"},"resumeEligible":resume,"canAutoNext":!live&&episode>0.0&&num(m,"duration")>0.0&&num(m,"duration")-num(m,"position")<=10.0&&num(m,"position")>0.0})
    })
}

pub(super) fn card_presentation(v: &Value) -> Result {
    Ok({
        let m = &v["item"];
        let queue = text(v, "context") == "queue";
        let live = text(m, "type") == "live";
        let episode = num(m, "episode") > 0.0 || text(m, "type") == "episode";
        let presentation = normalize("presentation", m)?;
        let candidates: &[(&str, &str)] = if live {
            &[("poster", "logo")]
        } else if queue && episode {
            // A series poster is not an episode still. An unavailable still
            // can use an explicitly known landscape, never a portrait crop.
            &[("thumbnail", "episode"), ("background", "landscape")]
        } else {
            &[
                ("background", "landscape"),
                ("thumbnail", "landscape"),
                ("poster", "poster"),
            ]
        };
        let (art, role) = candidates
            .iter()
            .find_map(|(key, role)| {
                let art = image(m, key);
                let failed = v["failedImages"]
                    .as_array()
                    .is_some_and(|urls| urls.contains(&art));
                (!art.is_null() && !failed).then_some((art, *role))
            })
            .unwrap_or((Value::Null, "none"));
        let status = match text(m, "queueStatus") {
            "next" => "Play next episode".to_owned(),
            "caught_up" => "You're caught up".to_owned(),
            "upcoming" => "Next episode coming soon".to_owned(),
            "pending" | "unavailable" => "Find next episode".to_owned(),
            _ if !live && num(m, "position") > 0.0 => {
                let seconds = num(m, "position").floor() as u64;
                format!("Resume at {}:{:02}", seconds / 60, seconds % 60)
            }
            _ => String::new(),
        };
        let mut context = Vec::new();
        if episode {
            context.push(format!(
                "S{:.0} · E{:.0}",
                num(m, "season"),
                num(m, "episode")
            ));
            if !text(m, "episodeTitle").is_empty() {
                context.push(text(m, "episodeTitle").to_owned());
            }
        }
        if !status.is_empty() {
            context.push(status);
        }
        if context.is_empty() && !live {
            if num(m, "year") > 0.0 {
                context.push(format!("{:.0}", num(m, "year")));
            }
            if !text(m, "runtime").is_empty() {
                context.push(text(m, "runtime").to_owned());
            }
            context.extend(
                m["genres"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .take(2)
                    .map(str::to_owned),
            );
        }
        let action = if live {
            "play"
        } else if !queue {
            "details"
        } else if matches!(
            text(m, "queueStatus"),
            "caught_up" | "upcoming" | "pending" | "unavailable"
        ) {
            "episodes"
        } else {
            text(&presentation, "primaryAction")
        };
        let label = match action {
            "details" => "Details",
            "episodes" => "Episodes",
            _ => text(&presentation, "primaryActionLabel"),
        };
        json!({"image":art,"imageRole":role,"title":m["name"],"subtitle":context.join(" · "),
                "progress":if !live && num(m,"duration")>0.0 { presentation["progress"].clone() } else { Value::Null },
                "primaryAction":action,"primaryActionLabel":label})
    })
}
