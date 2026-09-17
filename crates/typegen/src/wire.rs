//! Serde JSON declarations from Crux's Facet registry (not foreign object layout).
use facet_generate::{
    Registry,
    reflection::format::{
        ContainerFormat as C, EnumTagging, Format as F, Named, VariantFormat as V,
    },
};
fn ty(f: &F) -> String {
    match f {
        F::Unit => "null".into(),
        F::Bool => "boolean".into(),
        F::Str | F::Char | F::Uuid => "string".into(),
        F::TypeName(n) => n.name.clone(),
        F::Bytes => "number[]".into(),
        F::Option(f) => format!("{} | null", ty(f)),
        F::Seq(f) | F::Set(f) => format!("ReadonlyArray<{}>", ty(f)),
        F::Map { value, .. } => format!("{{ [key: string]: {} }}", ty(value)),
        F::Tuple(fs) => format!("[{}]", fs.iter().map(ty).collect::<Vec<_>>().join(", ")),
        F::TupleArray { content, size } => format!("[{}]", vec![ty(content); *size].join(", ")),
        F::I8
        | F::I16
        | F::I32
        | F::I64
        | F::I128
        | F::U8
        | F::U16
        | F::U32
        | F::U64
        | F::U128
        | F::F32
        | F::F64 => "number".into(),
        _ => panic!("Unsupported wire format {f:?}"),
    }
}
fn fields(fs: &[Named<F>], optional: bool) -> String {
    format!(
        "{{ {} }}",
        fs.iter()
            .map(|f| {
                if optional && let F::Option(inner) = &f.value {
                    return format!("readonly {}?: {}", f.name, ty(inner));
                }
                format!("readonly {}: {}", f.name, ty(&f.value))
            })
            .collect::<Vec<_>>()
            .join("; ")
    )
}
fn payload(v: &V) -> String {
    match v {
        V::Unit => "null".into(),
        V::NewType(f) => ty(f),
        V::Tuple(fs) => format!("[{}]", fs.iter().map(ty).collect::<Vec<_>>().join(", ")),
        V::Struct(fs) => fields(fs, false),
        _ => panic!("Unresolved variant"),
    }
}
pub fn generate(registry: &Registry) -> String {
    let mut out = String::from(
        "// Generated from Rust by viptv-typegen. Serde JSON bridge representation.\n",
    );
    for (name, container) in registry {
        let name = &name.name;
        let optional = matches!(
            name.as_str(),
            "Catalog"
                | "CatalogExtra"
                | "MediaItem"
                | "MediaSource"
                | "MediaTrack"
                | "DiscoverPage"
                | "Profile"
                | "PlaybackSession"
                | "PlaybackAuthorization"
        );
        let value = match container {
            C::UnitStruct(_) => "null".into(),
            C::NewTypeStruct(f, _) => ty(f),
            C::TupleStruct(fs, _) => {
                format!("[{}]", fs.iter().map(ty).collect::<Vec<_>>().join(", "))
            }
            C::Struct(fs, _) => fields(fs, optional),
            C::Enum(variants, tagging, _) => variants
                .values()
                .map(|v| {
                    // JsonValue is explicitly untagged in dto.rs; Facet's format AST
                    // currently omits that attribute, unlike internal/adjacent tags.
                    if name == "JsonValue" {
                        return payload(&v.value);
                    }
                    match tagging {
                        EnumTagging::External => {
                            if matches!(v.value, V::Unit) {
                                format!("{:?}", v.name)
                            } else {
                                format!("{{ readonly {}: {} }}", v.name, payload(&v.value))
                            }
                        }
                        EnumTagging::Internal { tag } => {
                            format!("{{ readonly {tag}: {:?} }} & {}", v.name, payload(&v.value))
                        }
                        EnumTagging::Adjacent { tag, content } => format!(
                            "{{ readonly {tag}: {:?}; readonly {content}: {} }}",
                            v.name,
                            payload(&v.value)
                        ),
                    }
                })
                .collect::<Vec<_>>()
                .join(" | "),
        };
        out.push_str(&format!("export type {name} = {value};\n"));
    }
    out
}
