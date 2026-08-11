#[derive(Debug, Clone, PartialEq)]
pub struct ChannelDef {
    pub id: u8,
    pub unit: String,
    pub scale: f64,
    pub offset: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageGroup {
    pub name: String,
    pub entity: u8,
    pub group: u8,
    pub channels: Vec<ChannelDef>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigurationCatalog {
    pub spec_format_version: u32,
    pub groups: Vec<MessageGroup>,
}

pub fn parse_configuration_ini(text: &str) -> ConfigurationCatalog {
    let mut catalog = ConfigurationCatalog::default();
    let mut current_group: Option<MessageGroup> = None;
    let mut current_channel: Option<(String, ChannelDef)> = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(value) = line.strip_prefix("SpecFormatVersion=") {
            catalog.spec_format_version = value.parse().unwrap_or(0);
            continue;
        }
        if line.starts_with("[[GRP_") && line.ends_with(']') {
            flush_channel(&mut current_group, &mut current_channel);
            flush_group(&mut catalog, &mut current_group);
            let name = line
                .trim_start_matches("[[")
                .trim_end_matches(']')
                .to_string();
            current_group = Some(MessageGroup {
                name,
                entity: 0,
                group: 0,
                channels: Vec::new(),
            });
            continue;
        }
        if line.starts_with("[CHN_") && line.ends_with(']') {
            flush_channel(&mut current_group, &mut current_channel);
            let name = line
                .trim_start_matches('[')
                .trim_end_matches(']')
                .to_string();
            current_channel = Some((
                name,
                ChannelDef {
                    id: 0,
                    unit: String::new(),
                    scale: 1.0,
                    offset: 0.0,
                },
            ));
            continue;
        }
        if let Some((_, channel)) = current_channel.as_mut() {
            if let Some(value) = line.strip_prefix("Id=") {
                channel.id = value.parse().unwrap_or(0);
            } else if let Some(value) = line.strip_prefix("Unit=") {
                channel.unit = value.to_string();
            } else if let Some(value) = line.strip_prefix("P1=") {
                channel.scale = value.parse().unwrap_or(1.0);
            } else if let Some(value) = line.strip_prefix("P2=") {
                channel.offset = value.parse().unwrap_or(0.0);
            }
        }
        if let Some(group) = current_group.as_mut() {
            if let Some(value) = line.strip_prefix("Id=") {
                if current_channel.is_none() {
                    group.entity = value.parse().unwrap_or(0);
                }
            } else if let Some(value) = line.strip_prefix("GId=") {
                group.group = value.parse().unwrap_or(0);
            }
        }
    }
    flush_channel(&mut current_group, &mut current_channel);
    flush_group(&mut catalog, &mut current_group);
    catalog
}

fn flush_group(catalog: &mut ConfigurationCatalog, group: &mut Option<MessageGroup>) {
    if let Some(group) = group.take() {
        catalog.groups.push(group);
    }
}

fn flush_channel(group: &mut Option<MessageGroup>, channel: &mut Option<(String, ChannelDef)>) {
    if let (Some(group), Some((_, channel_def))) = (group.as_mut(), channel.take()) {
        group.channels.push(channel_def);
    }
}

#[cfg(test)]
mod tests {
    use super::parse_configuration_ini;

    #[test]
    fn parses_spec_format_and_group() {
        let text = "SpecFormatVersion=1\n[[GRP_RealTimeData]]\nId=0\nGId=0\n[CHN_CountRate]\nId=1\nUnit=cps\nP1=1\nP2=0\n";
        let catalog = parse_configuration_ini(text);
        assert_eq!(catalog.spec_format_version, 1);
        assert_eq!(catalog.groups.len(), 1);
        assert_eq!(catalog.groups[0].entity, 0);
        assert_eq!(catalog.groups[0].group, 0);
        assert_eq!(catalog.groups[0].channels.len(), 1);
        assert_eq!(catalog.groups[0].channels[0].unit, "cps");
    }
}
