#[path = "../wire.rs"]
mod wire;
use crux_core::type_generation::facet::TypeRegistry;
use facet_generate::reflection::format::{ContainerFormat as C, Format as F, VariantFormat as V};
use viptv_engine::playback::{HistoryEntry, PlaybackRecord};
use viptv_engine::stremio::{SourceCandidate, SubtitleSnapshot};
use viptv_engine::{capabilities::*, identity::*, presentation::*, profile::*, query::*};

fn ty(f: &F) -> String {
    match f {
        F::Str | F::Char => "String".into(),
        F::Bool => "Boolean".into(),
        F::TypeName(n) => n.name.clone(),
        F::Option(v) => format!("{}?", ty(v)),
        F::Seq(v) => format!("List<{}>", ty(v)),
        F::F32 | F::F64 => "Double".into(),
        F::U64 | F::I64 | F::U32 => "Long".into(),
        F::U8 | F::U16 | F::I8 | F::I16 | F::I32 => "Int".into(),
        other => panic!("Unsupported engine field: {other:?}"),
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = TypeRegistry::new()
        .register_type::<PluginDescriptor>()?
        .register_type::<CapabilityState>()?
        .register_type::<ProfileConfiguration>()?
        .register_type::<IntegrationConnection>()?
        .register_type::<HomeSnapshot>()?
        .register_type::<viptv_engine::live::LivePage>()?
        .register_type::<viptv_engine::live::LiveGuide>()?
        .register_type::<viptv_engine::live::LiveQuery>()?
        .register_type::<DetailSnapshot>()?
        .register_type::<CatalogPage>()?
        .register_type::<viptv_engine::library::LibrarySnapshot>()?
        .register_type::<viptv_engine::library::LibraryImportStatus>()?
        .register_type::<viptv_engine::library::LibraryMembershipSnapshot>()?
        .register_type::<SourceCandidate>()?
        .register_type::<viptv_engine::continuation::ContinuationPage>()?
        .register_type::<viptv_engine::continuation::NextEpisodeRequest>()?
        .register_type::<viptv_engine::continuation::NextEpisodeResult>()?
        .register_type::<viptv_engine::continuation::QueueVisibilityRequest>()?
        .register_type::<viptv_engine::continuation::QueueVisibilityReceipt>()?
        .register_type::<SubtitleSnapshot>()?
        .register_type::<PlaybackRecord>()?
        .register_type::<HistoryEntry>()?
        .register_type::<QueryControl>()?
        .register_type::<QueryValue>()?
        .register_type::<MappingEvidence>()?
        .build()?
        .registry();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../generated/engine");
    std::fs::create_dir_all(&root)?;
    std::fs::write(root.join("wire.ts"), wire::generate(&registry))?;
    let mut kotlin = String::from(
        "// Generated from the Rust engine contracts.\npackage org.viptv.engine.wire\n\nimport kotlinx.serialization.Serializable\nimport kotlinx.serialization.SerialName\n\n",
    );
    for (name, container) in &registry {
        match container {
            C::Struct(fields, _) => {
                kotlin.push_str(&format!("@Serializable\ndata class {}(\n", name.name));
                for f in fields {
                    let default = if matches!(f.value, F::Option(_)) {
                        " = null"
                    } else if (name.name=="HomeBrowseAction"&&f.name=="query") || (name.name=="SourcePresentation"&&f.name=="audio_codecs") || (name.name=="LibraryStatistics"&&f.name=="provenance") || (name.name=="LiveChannel"&&f.name=="group_ids") || (name.name=="SourceCandidate" && matches!(f.name.as_str(),"audio_languages"|"dynamic_range"|"audio_flags")) || (name.name=="DetailSnapshot" && matches!(f.name.as_str(),"recommendations"|"seasons")) || (name.name=="HomeSnapshot" && f.name=="featured") || (name.name=="LibrarySnapshot" && matches!(f.name.as_str(),"controls"|"sources"|"statistics")) || (name.name=="CardMetadata" && matches!(f.name.as_str(),"genres"|"alternate_titles")) || (name.name=="LibraryMembershipSnapshot" && matches!(f.name.as_str(),"scoped_identities"|"legacy_identities")) {
                        " = emptyList()"
                    } else if name.name == "SourceCandidate" && f.name == "three_d" {
                        " = false"
                    } else {
                        ""
                    };
                    kotlin.push_str(&format!(
                        "    val {}: {}{},\n",
                        f.name,
                        ty(&f.value),
                        default
                    ));
                }
                kotlin.push_str(")\n\n");
            }
            C::Enum(variants, _, _) => {
                assert!(variants.values().all(|v| matches!(v.value, V::Unit)));
                kotlin.push_str(&format!("@Serializable\nenum class {} {{\n", name.name));
                for v in variants.values() {
                    kotlin.push_str(&format!(
                        "    @SerialName({:?}) {},\n",
                        v.name,
                        v.name.to_uppercase()
                    ));
                }
                kotlin.push_str("}\n\n");
            }
            _ => return Err("Unsupported engine contract".into()),
        }
    }
    std::fs::write(root.join("EngineWire.kt"), kotlin)?;
    println!("Generated engine Kotlin and TypeScript contracts");
    Ok(())
}
