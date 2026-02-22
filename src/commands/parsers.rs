use anyhow::bail;

pub fn parse_bool(value: &str) -> anyhow::Result<bool> {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => bail!(
            "Invalid boolean '{}'. Use true/false, 1/0, yes/no, or on/off.",
            value
        ),
    }
}

pub fn parse_u32(value: &str) -> anyhow::Result<u32> {
    value
        .parse::<u32>()
        .map_err(|_| anyhow::anyhow!("Invalid u32 value '{}'", value))
}

pub fn parse_i32(value: &str) -> anyhow::Result<i32> {
    value
        .parse::<i32>()
        .map_err(|_| anyhow::anyhow!("Invalid i32 value '{}'", value))
}

pub fn parse_f32(value: &str) -> anyhow::Result<f32> {
    value
        .parse::<f32>()
        .map_err(|_| anyhow::anyhow!("Invalid f32 value '{}'", value))
}

pub fn parse_u64(value: &str) -> anyhow::Result<u64> {
    value
        .parse::<u64>()
        .map_err(|_| anyhow::anyhow!("Invalid u64 value '{}'", value))
}

pub fn format_uptime(seconds: u32, include_seconds: bool) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let mins = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        if include_seconds {
            format!("{}h {}m {}s", hours, mins, secs)
        } else {
            format!("{}h {}m", hours, mins)
        }
    } else if include_seconds {
        if mins > 0 {
            format!("{}m {}s", mins, secs)
        } else {
            format!("{}s", secs)
        }
    } else {
        format!("{}m", mins)
    }
}

pub fn hex_decode(hex: &str) -> anyhow::Result<Vec<u8>> {
    if hex.is_empty() {
        return Ok(vec![]);
    }
    if !hex.len().is_multiple_of(2) {
        bail!("Hex string must have even length");
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|_| anyhow::anyhow!("Invalid hex character in '{}'", &hex[i..i + 2]))
        })
        .collect()
}

pub fn base64_url_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    let mut result = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let chunks = bytes.chunks(3);

    for chunk in chunks {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;

        result.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        result.push(TABLE[((n >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(TABLE[((n >> 6) & 0x3F) as usize] as char);
        }
        if chunk.len() > 2 {
            result.push(TABLE[(n & 0x3F) as usize] as char);
        }
    }

    result
}

pub fn base64_url_decode(input: &str) -> anyhow::Result<Vec<u8>> {
    let input = input.replace('-', "+").replace('_', "/");

    let padded = match input.len() % 4 {
        2 => format!("{}==", input),
        3 => format!("{}=", input),
        _ => input,
    };

    let mut result = Vec::new();
    let chars: Vec<u8> = padded.bytes().collect();

    for chunk in chars.chunks(4) {
        if chunk.len() < 4 {
            break;
        }

        let a = b64_val(chunk[0])?;
        let b = b64_val(chunk[1])?;
        let c = b64_val(chunk[2])?;
        let d = b64_val(chunk[3])?;

        result.push((a << 2) | (b >> 4));
        if chunk[2] != b'=' {
            result.push(((b & 0x0F) << 4) | (c >> 2));
        }
        if chunk[3] != b'=' {
            result.push(((c & 0x03) << 6) | d);
        }
    }

    Ok(result)
}

fn b64_val(c: u8) -> anyhow::Result<u8> {
    match c {
        b'A'..=b'Z' => Ok(c - b'A'),
        b'a'..=b'z' => Ok(c - b'a' + 26),
        b'0'..=b'9' => Ok(c - b'0' + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        b'=' => Ok(0),
        _ => bail!("Invalid base64 character: {}", c as char),
    }
}

pub fn extract_meshtastic_url_payload(url: &str) -> anyhow::Result<String> {
    if let Some(payload) = url.strip_prefix("https://meshtastic.org/e/#") {
        Ok(payload.to_string())
    } else if let Some(payload) = url.strip_prefix("http://meshtastic.org/e/#") {
        Ok(payload.to_string())
    } else if let Some(payload) = url.strip_prefix("meshtastic://") {
        Ok(payload.to_string())
    } else if let Some(payload) = url.strip_prefix('#') {
        Ok(payload.to_string())
    } else {
        Ok(url.to_string())
    }
}

pub fn find_next_free_channel_index(
    channels: &[meshtastic::protobufs::Channel],
) -> anyhow::Result<i32> {
    use meshtastic::protobufs::channel;
    for i in 1..=7 {
        let is_used = channels
            .iter()
            .any(|c| c.index == i && c.role != channel::Role::Disabled as i32);
        if !is_used {
            return Ok(i);
        }
    }
    bail!("No free channel slots available (max 8 channels, indices 0-7)")
}

pub fn parse_enum_i32(value: &str, variants: &[(&str, i32)]) -> anyhow::Result<i32> {
    let upper = value.to_uppercase();
    for (name, val) in variants {
        if name.to_uppercase() == upper {
            return Ok(*val);
        }
    }
    // Also try parsing as a raw integer
    if let Ok(n) = value.parse::<i32>() {
        return Ok(n);
    }
    let names: Vec<&str> = variants.iter().map(|(n, _)| *n).collect();
    bail!(
        "Invalid value '{}'. Valid options: {}",
        value,
        names.join(", ")
    )
}
