mod kotlin;
mod wire;
use crux_core::type_generation::facet::TypeRegistry;
use facet_generate::reflection::format::{Format, FormatHolder};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../generated");
    let generator = TypeRegistry::new()
        .register_app::<viptv_core::Viptv>()?
        .register_type::<viptv_core::Session>()?
        .register_type::<viptv_core::dto::MediaPresentation>()?
        .register_type::<viptv_core::dto::CardPresentation>()?
        .register_type::<viptv_core::dto::SourcePresentation>()?
        .register_type::<viptv_core::dto::HomeActions>()?
        .register_type::<viptv_core::dto::EpisodeWatching>()?
        .register_type::<viptv_core::dto::PhonePresentation>()?
        .register_type::<viptv_core::dto::SourceRanks>()?
        .register_type::<viptv_core::dto::SourceProducerOutcome>()?
        .register_type::<viptv_core::dto::DiscoverPolicyProjection>()?
        .register_type::<viptv_core::dto::MetadataTarget>()?
        .register_type::<viptv_core::policy::shell_lifecycle::ForegroundAuthorityInput>()?
        .register_type::<viptv_core::policy::shell_lifecycle::ForegroundAuthorityDecision>()?
        .register_type::<viptv_core::policy::shell_lifecycle::HomeRevisionInput>()?
        .register_type::<viptv_core::policy::shell_lifecycle::HomeRevisionDecision>()?
        .register_type::<viptv_core::policy::shell_lifecycle::PreviewScopeInput>()?
        .register_type::<viptv_core::policy::shell_lifecycle::PreviewScopeDecision>()?
        .register_type::<viptv_core::policy::shell_lifecycle::PreviewInput>()?
        .register_type::<viptv_core::policy::shell_lifecycle::PreviewDecision>()?
        .register_type::<viptv_core::policy::shell_lifecycle::UpNextGateInput>()?
        .register_type::<viptv_core::policy::shell_lifecycle::UpNextGateDecision>()?
        .register_type::<viptv_core::policy::shell_lifecycle::CountdownInput>()?
        .register_type::<viptv_core::policy::shell_lifecycle::CountdownDecision>()?
        .register_type::<viptv_core::policy::shell_lifecycle::UpNextPlaybackInput>()?
        .register_type::<viptv_core::policy::shell_lifecycle::UpNextPlaybackDecision>()?
        .register_type::<viptv_core::policy::playback_control::PlaybackTimelineFacts>()?
        .register_type::<viptv_core::policy::playback_control::PlaybackTimelineProjection>()?
        .register_type::<viptv_core::policy::playback_control::PlaybackSeekFacts>()?
        .register_type::<viptv_core::policy::playback_control::PlaybackPauseFacts>()?
        .register_type::<viptv_core::policy::playback_control::PlaybackPauseDecision>()?
        .register_type::<viptv_core::policy::playback_control::PlaybackRecoveryFacts>()?
        .register_type::<viptv_core::policy::playback_control::PlaybackDeliveryFacts>()?
        .register_type::<viptv_core::policy::playback_control::PlaybackLeaseFacts>()?
        .register_type::<viptv_core::policy::playback_control::PlaybackLeaseDecision>()?
        .register_type::<viptv_core::policy::playback_control::PlaybackAuthorityFacts>()?
        .register_type::<viptv_core::policy::playback_control::PlaybackAuthorityBudget>()?
        .register_type::<viptv_core::policy::playback_control::PlaybackFailureFacts>()?
        .register_type::<viptv_core::policy::playback_control::PlaybackFailureDecision>()?
        .register_type::<viptv_core::policy::playback_control::LivePageValidationFacts>()?
        .register_type::<viptv_core::policy::playback_control::LivePageValidationDecision>()?
        .register_type::<viptv_core::dto::ApiRequest>()?
        .register_type::<viptv_core::vizio::VizioHttpMethod>()?
        .register_type::<viptv_core::vizio::VizioRequest>()?
        .register_type::<viptv_core::vizio::VizioRequestResult>()?
        .register_type::<viptv_core::vizio::VizioFailureKind>()?
        .register_type::<viptv_core::vizio::VizioFailure>()?
        .register_type::<viptv_core::vizio::VizioProtocolResponse>()?
        .register_type::<viptv_core::vizio::VizioResponseResult>()?
        .register_type::<viptv_core::vizio::VizioRemoteAction>()?
        .register_type::<viptv_core::vizio::VizioRemoteEvent>()?
        .register_type::<viptv_core::vizio::VizioRemoteKey>()?
        .register_type::<viptv_core::vizio::VizioPairingChallenge>()?
        .register_type::<viptv_core::vizio::VizioAppConfig>()?
        .register_type::<viptv_core::vizio::VizioInputInfo>()?
        .register_type::<viptv_core::vizio::VizioDiscoveryCandidate>()?
        .register_type::<viptv_core::vizio::VizioTransportSupport>()?
        .register_type::<viptv_core::vizio::VizioPlatformSupport>()?
        .register_type::<viptv_core::vizio::VizioControllerOutputKind>()?
        .register_type::<viptv_core::vizio::VizioControllerOutput>()?
        .register_type::<viptv_core::dto::Catalog>()?
        .register_type::<viptv_core::dto::MediaItem>()?
        .register_type::<viptv_core::dto::MediaSource>()?
        .register_type::<viptv_core::dto::SourcesPollStep>()?
        .register_type::<viptv_core::dto::PlaybackSession>()?
        .register_type::<viptv_core::dto::PlaybackLease>()?
        .register_type::<viptv_core::dto::PlaybackProtocol>()?
        .register_type::<viptv_core::native_torrent::NativeTorrentContext>()?
        .register_type::<viptv_core::native_torrent::NativeTorrentObservation>()?
        .register_type::<viptv_core::native_torrent::NativeTorrentClock>()?
        .register_type::<viptv_core::native_torrent::NativeTorrentState>()?
        .register_type::<viptv_core::native_torrent::NativeTorrentNegotiationFacts>()?
        .register_type::<viptv_core::native_torrent::NativeTorrentNegotiationDecision>()?
        .register_type::<viptv_core::native_torrent::NativeTorrentRecoveryFacts>()?
        .register_type::<viptv_core::native_torrent::NativeTorrentRecoveryDecision>()?
        .register_type::<viptv_core::dto::PlaybackV2Request>()?
        .register_type::<viptv_core::dto::DiscoverPage>()?
        .register_type::<viptv_core::dto::LiveCatalogPage>()?
        .register_type::<viptv_core::dto::LiveCatalogCategories>()?
        .build()?;
    let registry = generator.registry();
    std::fs::create_dir_all(root.join("typescript"))?;
    std::fs::write(root.join("typescript/wire.ts"), wire::generate(&registry))?;
    std::fs::create_dir_all(root.join("kotlin-wire"))?;
    std::fs::write(
        root.join("kotlin-wire/Wire.kt"),
        kotlin::generate(&registry),
    )?;
    // Declaration-only foreign types; live Kotlin uses the UniFFI JSON string API.
    let mut kotlin_registry = registry.clone();
    for container in kotlin_registry.values_mut() {
        container.visit_mut(&mut |format| {
            if matches!(format, Format::Bytes) {
                *format = Format::Seq(Box::new(Format::U8));
            }
            Ok(())
        })?;
    }
    facet_generate::generation::kotlin::Installer::new("org.viptv.core.types", root.join("kotlin"))
        .generate(&kotlin_registry)?;
    // Raw negotiation text may be a malicious reflected private response.
    // Keep declaration-only data classes redacted as well as the live wire DTO.
    let declaration = root.join("kotlin/org/viptv/core/types/Types.kt");
    let mut text = std::fs::read_to_string(&declaration)?;
    let start = text
        .find("data class NativeTorrentNegotiationFacts(")
        .ok_or("Missing negotiation declaration")?;
    let end = start
        + text[start..]
            .find("\n)\n")
            .ok_or("Missing negotiation declaration end")?
        + 2;
    text.insert_str(
        end,
        " { override fun toString(): String = \"NativeTorrentNegotiationFacts(<redacted>)\" }",
    );
    std::fs::write(declaration, text)?;
    // Generate without invoking an unrelated package manager or building JS.
    facet_generate::generation::typescript::Installer::new(
        "viptv_core_types",
        root.join("typescript"),
    )
    .generate(&registry)?;
    Ok(())
}
