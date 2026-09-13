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
        .register_type::<viptv_core::dto::SourcePresentation>()?
        .register_type::<viptv_core::dto::ApiRequest>()?
        .register_type::<viptv_core::dto::Catalog>()?
        .register_type::<viptv_core::dto::MediaItem>()?
        .register_type::<viptv_core::dto::MediaSource>()?
        .register_type::<viptv_core::dto::PlaybackSession>()?
        .register_type::<viptv_core::dto::DiscoverPage>()?
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
    // Generate without invoking an unrelated package manager or building JS.
    facet_generate::generation::typescript::Installer::new(
        "viptv_core_types",
        root.join("typescript"),
    )
    .generate(&registry)?;
    Ok(())
}
