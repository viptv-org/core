//! Persistent JSON-lines shell for WASI and the wasm2brs Roku experiment.
use std::io::{self, BufRead, Read, Write};
use viptv_core::{CoreBridge, CoreError};

// HTTP effects encode a <=2 MiB body as JSON bytes, which can need ~8 MiB.
const MAX_REQUEST_BYTES: u64 = 12 * 1024 * 1024;

fn dispatch(core: &mut CoreBridge, input: &[u8]) -> Result<String, CoreError> {
    let request: serde_json::Value =
        serde_json::from_slice(input).map_err(|_| CoreError::InvalidInput)?;
    let field = |key: &str| request.get(key).ok_or(CoreError::InvalidInput);
    let text = |key: &str| {
        request
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
            .ok_or(CoreError::InvalidInput)
    };
    match request.get("op").and_then(serde_json::Value::as_str) {
        Some("normalize") => viptv_core::normalize(
            text("kind")?,
            field("input")?.to_string(),
            request
                .get("origin")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        ),
        Some("vizio_platform_support") => Ok(viptv_core::vizio_platform_support(text("platform")?)),
        Some("update" | "process_event") => core.update(field("event")?.to_string()),
        Some("resolve" | "handle_response") => {
            let id = field("id")?
                .as_u64()
                .and_then(|id| u32::try_from(id).ok())
                .ok_or(CoreError::InvalidInput)?;
            core.resolve(id, field("result")?.to_string())
        }
        Some("view") => core.view(),
        Some("reset") => {
            *core = CoreBridge::new();
            core.view()
        }
        _ => Err(CoreError::InvalidInput),
    }
}

fn main() -> io::Result<()> {
    let mut core = CoreBridge::new();
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    let mut line = Vec::new();
    loop {
        line.clear();
        if input
            .by_ref()
            .take(MAX_REQUEST_BYTES + 1)
            .read_until(b'\n', &mut line)?
            == 0
        {
            break;
        }
        let oversized = line.len() as u64 > MAX_REQUEST_BYTES;
        let result = if oversized {
            Err(CoreError::InvalidInput)
        } else {
            dispatch(&mut core, &line)
        };
        match result {
            // Core outputs already are JSON: do not parse and serialize them twice.
            Ok(result) => writeln!(output, "{{\"ok\":true,\"result\":{result}}}")?,
            Err(error) => writeln!(
                output,
                "{}",
                serde_json::json!({"ok": false, "error": error.to_string()})
            )?,
        }
        // Resolve effects before requesting the next input; never wait for EOF.
        output.flush()?;
        if oversized {
            break;
        }
    }
    Ok(())
}
