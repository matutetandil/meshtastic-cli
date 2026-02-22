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
