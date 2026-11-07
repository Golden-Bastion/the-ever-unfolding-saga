use super::definitions::*;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

impl ContentLoadReport {
    fn with_content(content: CampaignContent) -> Self {
        Self {
            content,
            loaded_mods: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

pub fn load_campaign_content() -> CampaignContent {
    load_campaign_content_report().content
}

pub fn load_campaign_content_report() -> ContentLoadReport {
    let mut report = ContentLoadReport::with_content(default_campaign_content());
    let installed = crate::mod_manager::active_mods();
    let mut pending_mod_events = Vec::new();

    for item in installed.iter().filter(|item| item.enabled) {
        let manifest = item.manifest.clone();
        // Assets remain active even if this mod's optional content file is invalid.
        report.loaded_mods.push(manifest.clone());
        match load_mod_content(&item.directory, &manifest) {
            Ok(Some(mut mod_content)) => {
                pending_mod_events.push((
                    item.priority,
                    manifest.id.clone(),
                    std::mem::take(&mut mod_content.events),
                ));
                merge_campaign_content(&mut report.content, mod_content);
            }
            Ok(None) => {}
            Err(error) => report.warnings.push(format!(
                "could not load mod {} ({}) from {}: {}",
                manifest.id,
                manifest.name,
                item.directory.display(),
                error
            )),
        }
    }

    // Cross-mod references are checked only after all enabled mods have contributed
    // their locations, factions, and event IDs.
    merge_mod_events(
        &mut report.content,
        pending_mod_events,
        &mut report.warnings,
    );
    report.content.loaded_mods = report.loaded_mods.clone();
    report.warnings.extend(report.content.validate());
    report
}

fn load_content_file(path: &Path) -> io::Result<CampaignContent> {
    let data = fs::read(path)?;
    serde_json::from_slice(&data)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))
}

fn load_mod_content(
    mod_root: &Path,
    manifest: &ModManifest,
) -> io::Result<Option<CampaignContent>> {
    let Some(content_file) = manifest.content_file.as_deref() else {
        return Ok(None);
    };
    let relative = PathBuf::from(content_file);
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || !relative
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        || content_file.contains('\\')
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "unsafe mod content path",
        ));
    }
    let canonical_root = fs::canonicalize(mod_root)?;
    let canonical_path = fs::canonicalize(canonical_root.join(relative))?;
    if !canonical_path.starts_with(&canonical_root) || !canonical_path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "mod content path escapes its directory",
        ));
    }
    load_content_file(&canonical_path).map(Some)
}

fn merge_campaign_content(base: &mut CampaignContent, incoming: CampaignContent) {
    base.world.region = incoming.world.region;
    merge_vec_by_key(
        &mut base.world.locations,
        incoming.world.locations,
        |entry| entry.id.clone(),
    );
    merge_vec_by_key(&mut base.factions, incoming.factions, |entry| {
        entry.id.clone()
    });
    merge_vec_by_key(&mut base.npcs, incoming.npcs, |entry| entry.id.clone());
    merge_vec_by_key(&mut base.character_parts, incoming.character_parts, |entry| entry.id.clone());
    merge_vec_by_key(&mut base.quests, incoming.quests, |entry| entry.id.clone());
    merge_vec_by_key(&mut base.encounters, incoming.encounters, |entry| {
        entry.location_name.clone()
    });
    merge_vec_by_key(&mut base.atmospheres, incoming.atmospheres, |entry| {
        entry.location_name.clone()
    });
    merge_vec_by_key(&mut base.item_visuals, incoming.item_visuals, |entry| {
        entry.item_name.clone()
    });
}

fn merge_mod_events(
    base: &mut CampaignContent,
    mut mod_events: Vec<(i32, String, Vec<EventContent>)>,
    warnings: &mut Vec<String>,
) {
    // Match asset precedence: highest priority wins, then reverse lexical mod ID.
    mod_events.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| right.1.cmp(&left.1)));

    let locations: HashSet<&str> = base
        .world
        .locations
        .iter()
        .map(|entry| entry.name.as_str())
        .collect();
    let factions: HashSet<&str> = base
        .factions
        .iter()
        .map(|entry| entry.name.as_str())
        .collect();
    let existing_ids: HashSet<String> = base.events.iter().map(|event| event.id.clone()).collect();

    let mut seen_ids = existing_ids.clone();
    let mut known_ids = existing_ids.clone();
    let mut candidates = Vec::new();

    for (_, mod_id, events) in mod_events {
        for event in events {
            if !seen_ids.insert(event.id.clone()) {
                warnings.push(format!(
                    "rejecting event '{}' from mod {}: duplicate event ID",
                    event.id, mod_id
                ));
                continue;
            }
            known_ids.insert(event.id.clone());
            candidates.push((mod_id.clone(), event));
        }
    }

    let mut pending = Vec::new();
    for (mod_id, event) in candidates {
        let issues = validate_event_content(&event, &locations, &factions, &known_ids);
        if issues.is_empty() {
            pending.push((mod_id, event));
        } else {
            warnings.push(format!(
                "rejecting event '{}' from mod {}: {}",
                event.id,
                mod_id,
                issues.join("; ")
            ));
        }
    }

    // References to events that fail validation are pruned in dependency order.
    loop {
        let available: HashSet<String> = existing_ids
            .iter()
            .cloned()
            .chain(pending.iter().map(|(_, event)| event.id.clone()))
            .collect();
        let mut removed = false;
        let mut next = Vec::new();

        for (mod_id, event) in pending {
            let prior_id = event
                .conditions
                .as_ref()
                .and_then(|conditions| conditions.prior_event_id.as_deref());
            if let Some(prior_id) = prior_id.filter(|id| !available.contains(*id)) {
                warnings.push(format!(
                    "rejecting event '{}' from mod {}: prior event '{}' was rejected or unavailable",
                    event.id, mod_id, prior_id
                ));
                removed = true;
            } else {
                next.push((mod_id, event));
            }
        }

        pending = next;
        if !removed {
            break;
        }
    }

    base.events
        .extend(pending.into_iter().map(|(_, event)| event));
}

fn validate_event_content(
    event: &EventContent,
    locations: &HashSet<&str>,
    factions: &HashSet<&str>,
    known_ids: &HashSet<String>,
) -> Vec<String> {
    let mut issues = Vec::new();
    if event.id.trim().is_empty() {
        issues.push("empty id".into());
    }
    if event.trigger.trim().is_empty() {
        issues.push("empty trigger".into());
    }
    if event.weight == 0 {
        issues.push("zero weight".into());
    }
    if let Some(chance) = event.chance_percent {
        if chance > 100 {
            issues.push(format!("invalid chance {}", chance));
        }
    }
    if event.effects.is_empty() {
        issues.push("no effects".into());
    }
    if let Some(conditions) = &event.conditions {
        for location in &conditions.locations {
            if !locations.contains(location.as_str()) {
                issues.push(format!("unknown location {}", location));
            }
        }
        if let (Some(min_day), Some(max_day)) = (conditions.min_day, conditions.max_day) {
            if min_day > max_day {
                issues.push("min_day greater than max_day".into());
            }
        }
        if let Some(prior) = conditions.prior_event_id.as_deref() {
            if !known_ids.contains(prior) {
                issues.push(format!("unknown prior event id {}", prior));
            }
        }
        if let Some(faction) = conditions.faction_name.as_deref() {
            if !factions.contains(faction) {
                issues.push(format!("unknown faction {}", faction));
            }
        }
        if (conditions.min_reputation.is_some() || conditions.max_reputation.is_some())
            && conditions.faction_name.is_none()
        {
            issues.push("reputation condition requires faction_name".into());
        }
        if let (Some(min), Some(max)) = (conditions.min_reputation, conditions.max_reputation) {
            if min > max {
                issues.push("min_reputation greater than max_reputation".into());
            }
        }
        if conditions
            .required_item_name
            .as_deref()
            .map(|name| name.trim().is_empty())
            .unwrap_or(false)
        {
            issues.push("required_item_name cannot be empty".into());
        }
        if conditions
            .required_condition_name
            .as_deref()
            .map(|name| name.trim().is_empty())
            .unwrap_or(false)
        {
            issues.push("required_condition_name cannot be empty".into());
        }
    }
    issues
}

fn merge_vec_by_key<T, F>(base: &mut Vec<T>, incoming: Vec<T>, key_fn: F)
where
    T: Clone,
    F: Fn(&T) -> String,
{
    for item in incoming {
        let key = key_fn(&item);
        if let Some(position) = base.iter().position(|existing| key_fn(existing) == key) {
            base[position] = item;
        } else {
            base.push(item);
        }
    }
}

fn default_campaign_content() -> CampaignContent {
    serde_json::from_str(include_str!("../../data/base_content.json"))
        .expect("embedded base content JSON must remain valid")
}

#[cfg(test)]
#[path = "../../tests/unit/content/loader.rs"]
mod tests;
