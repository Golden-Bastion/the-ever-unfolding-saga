use crate::model::{Condition, GameState};

pub(crate) fn advance_time(state: &mut GameState, amount: u32) {
    let total = u64::from(state.world.time_points) + u64::from(amount);
    let elapsed_days = u32::try_from(total / 12).unwrap_or(u32::MAX);
    state.world.day = state.world.day.saturating_add(elapsed_days);
    state.world.time_points = (total % 12) as u32;
    for condition in &mut state.character.conditions {
        condition.remaining = condition.remaining.saturating_sub(amount);
    }
    state
        .character
        .conditions
        .retain(|condition| condition.remaining > 0);
    if amount > 0 && state.character.hp <= state.character.max_hp / 3 && state.character.alive {
        add_or_refresh_condition(
            &mut state.character.conditions,
            Condition::new("Wounded", 3, -1),
        );
    }
    if amount > 0 {
        crate::procedural_opportunities::evolve_generated_world(state);
    }
}

pub(crate) fn add_or_refresh_condition(conditions: &mut Vec<Condition>, condition: Condition) {
    if let Some(existing) = conditions
        .iter_mut()
        .find(|current| current.name == condition.name)
    {
        existing.remaining = existing.remaining.max(condition.remaining);
        existing.penalty = condition.penalty;
        existing.bonus = condition.bonus;
    } else {
        conditions.push(condition);
    }
}

pub(crate) fn remove_condition(conditions: &mut Vec<Condition>, name: &str) {
    conditions.retain(|condition| condition.name != name);
}

pub(crate) fn is_night(points: u32) -> bool {
    matches!(points % 12, 0 | 1 | 10 | 11)
}

#[cfg(test)]
#[path = "../../tests/unit/game/state_effects.rs"]
mod tests;
