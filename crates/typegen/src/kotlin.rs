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
        F::I64 | F::U64 => "Long".into(),
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
                    | "MediaSource"
                    | "MediaTrack"
                    | "MediaPresentation"
                    | "PlaybackSession"
                    | "Catalog"
                    | "CatalogExtra"
                    | "DiscoverPage"
                    | "ApiRequest"
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
