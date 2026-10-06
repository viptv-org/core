use facet_generate::{
    Registry,
    reflection::format::{ContainerFormat as C, Format as F},
};
fn ty(f: &F) -> String {
    match f {
        F::TypeName(n) if n.name == "JsonValue" => "JsonElement".into(),
        F::TypeName(n) => n.name.clone(),
        F::Str | F::Char | F::Uuid => "String".into(),
        F::Bool => "Boolean".into(),
        F::Option(f) => format!("{}?", ty(f)),
        F::Seq(f) | F::Set(f) => format!("List<{}>", ty(f)),
        F::Map { value, .. } => format!("Map<String, {}>", ty(value)),
        F::F32 | F::F64 => "Double".into(),
        F::I64 | F::U32 | F::U64 => "Long".into(),
        _ => "Int".into(),
    }
}
pub fn generate(registry: &Registry) -> String {
    let mut s = String::from(
        "// Generated from Rust Facet registry; do not edit.\npackage org.viptv.core.wire\nimport kotlinx.serialization.*\nimport kotlinx.serialization.json.*\n\nobject CoreJson { val codec = Json { ignoreUnknownKeys = true; explicitNulls = false }\n inline fun <reified T> decode(value: String): T = codec.decodeFromString(value)\n inline fun <reified T> encode(value: T): String = codec.encodeToString(value)\n}\n",
    );
    // Wire enums with only unit variants serialize as their declared strings. Complex Crux
    // events remain the existing string bridge; DTOs use generated serializable classes.
    for (name, c) in registry {
        if let C::Enum(variants, _, _) = c
            && variants.values().all(|v| {
                matches!(
                    v.value,
                    facet_generate::reflection::format::VariantFormat::Unit
                )
            })
        {
            s.push_str(&format!(
                "@Serializable enum class {} {{ {} }}\n",
                name.name,
                variants
                    .values()
                    .map(|v| format!("@SerialName({:?}) {}", v.name, v.name.to_uppercase()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    for (name, c) in registry {
        if let C::Struct(fields, _) = c
            && matches!(
                name.name.as_str(),
                "MediaItem"
                    | "HomeActions"
                    | "EpisodeWatching"
                    | "PhonePresentation"
                    | "SourceRank"
                    | "SourceRanks"
                    | "SourceProducerOutcome"
                    | "DiscoverPolicyProjection"
                    | "ForegroundAuthorityInput"
                    | "HomeRevisionInput"
                    | "PreviewScopeInput"
                    | "PreviewScopeDecision"
                    | "PreviewInput"
                    | "UpNextPlaybackInput"
                    | "UpNextPlaybackDecision"
                    | "UpNextGateInput"
                    | "UpNextGateDecision"
                    | "CountdownInput"
                    | "CountdownDecision"
                    | "PlaybackTimelineFacts"
                    | "PlaybackTimelineProjection"
                    | "PlaybackSeekFacts"
                    | "PlaybackPauseFacts"
                    | "PlaybackPauseDecision"
                    | "PlaybackRecoveryFacts"
                    | "PlaybackDeliveryFacts"
                    | "PlaybackLeaseFacts"
                    | "PlaybackAuthorityFacts"
                    | "PlaybackAuthorityBudget"
                    | "PlaybackFailureFacts"
                    | "PlaybackFailureDecision"
                    | "LivePageValidationFacts"
                    | "SourceFailure"
                    | "SourcesPollState"
                    | "SourcesPollStep"
                    | "MediaSource"
                    | "MediaTrack"
                    | "MediaPresentation"
                    | "CardPresentation"
                    | "SourcePresentation"
                    | "PlaybackSession"
                    | "PlaybackLease"
                    | "PlaybackProtocol"
                    | "PlaybackV2Request"
                    | "PlaybackClient"
                    | "PlaybackAuthorization"
                    | "Catalog"
                    | "CatalogExtra"
                    | "DiscoverPage"
                    | "LiveCatalogPage"
                    | "LiveCatalogCategory"
                    | "LiveCatalogCategories"
                    | "ApiRequest"
                    | "VizioRequest"
                    | "VizioFailure"
                    | "VizioProtocolResponse"
                    | "VizioRemoteEvent"
                    | "VizioPairingChallenge"
                    | "VizioAppConfig"
                    | "VizioInputInfo"
                    | "VizioDiscoveryCandidate"
                    | "VizioPlatformSupport"
                    | "VizioControllerOutput"
                    | "Profile"
                    | "Identity"
                    | "Account"
                    | "Session"
                    | "ViewModel"
            )
        {
            s.push_str(&format!(
                "@Serializable data class {}(\n{}\n)\n",
                name.name,
                fields
                    .iter()
                    .map(|f| {
                        let default = match &f.value {
                            F::Option(_) => " = null",
                            F::Seq(_) => " = emptyList()",
                            F::Map { .. } => " = emptyMap()",
                            _ => "",
                        };
                        format!("    val `{}`: {}{}", f.name, ty(&f.value), default)
                    })
                    .collect::<Vec<_>>()
                    .join(",\n")
            ));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::generate;
    use crux_core::type_generation::facet::TypeRegistry;

    #[test]
    fn registered_policy_dtos_generate_serializable_kotlin_structs() {
        let generator = TypeRegistry::new()
            .register_type::<viptv_core::dto::HomeActions>()
            .unwrap()
            .register_type::<viptv_core::dto::SourceRanks>()
            .unwrap()
            .register_type::<viptv_core::policy::shell_lifecycle::ForegroundAuthorityInput>()
            .unwrap()
            .register_type::<viptv_core::policy::shell_lifecycle::UpNextPlaybackDecision>()
            .unwrap()
            .register_type::<viptv_core::policy::shell_lifecycle::CountdownDecision>()
            .unwrap()
            .register_type::<viptv_core::policy::playback_control::PlaybackTimelineFacts>()
            .unwrap()
            .build()
            .unwrap();
        let output = generate(&generator.registry());
        for name in [
            "HomeActions",
            "SourceRank",
            "SourceRanks",
            "ForegroundAuthorityInput",
            "UpNextPlaybackDecision",
            "CountdownDecision",
            "PlaybackTimelineFacts",
        ] {
            assert!(
                output.contains(&format!("@Serializable data class {name}(")),
                "Missing {name}"
            );
        }
        assert!(output.contains("val `remainingMillis`: Long"));
        assert!(output.contains("val `resumeAwaitingKey`: String? = null"));
    }
}
