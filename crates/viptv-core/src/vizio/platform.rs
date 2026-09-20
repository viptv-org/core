use super::failure;
use super::types::{
    VizioDiscoveryCandidate, VizioFailure, VizioFailureKind, VizioPlatformSupport,
    VizioTransportSupport,
};

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
