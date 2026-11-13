use bevy::prelude::*;
use bevy::remote::BrpResult;
use serde::Deserialize;
use serde_json::{Value, json};

use super::json::{debug_name, entity_id, invalid_params, parse_entity, parse_params};
use crate::prelude::*;

#[derive(Deserialize)]
struct ExplainParams {
    entity: Option<u64>,
}

/// `game/explain {entity?}`: how each of a brain's behaviors scored at its last decision, with
/// every consideration's input and score, or why the behavior sat out. Without `entity`, returns
/// a list with every brain. The first call on a brain starts tracing it and returns no behaviors;
/// it stays traced until it despawns.
pub(super) fn explain_brains(
    In(params): In<Option<Value>>,
    mut commands: Commands,
    brains: Query<(Entity, &Brain, Option<&ScoreTrace>)>,
) -> BrpResult {
    let ExplainParams { entity } = parse_params(params)?;

    let mut explain_or_start_tracing = |entity, brain, score_trace: Option<&ScoreTrace>| {
        if let Some(score_trace) = score_trace {
            explain_brain(entity, brain, score_trace)
        } else {
            commands.entity(entity).insert(ScoreTrace::default());
            json!({ "entity": entity_id(entity), "tracing_started": true, "behaviors": [] })
        }
    };

    let Some(entity) = entity else {
        let explanations: Vec<Value> = brains
            .iter()
            .map(|(entity, brain, score_trace)| {
                explain_or_start_tracing(entity, brain, score_trace)
            })
            .collect();
        return Ok(json!(explanations));
    };

    let (entity, brain, score_trace) = brains
        .get(parse_entity(entity)?)
        .map_err(|_| invalid_params(format!("entity {entity} has no brain")))?;
    Ok(explain_or_start_tracing(entity, brain, score_trace))
}

#[expect(
    clippy::redundant_closure_for_method_calls,
    reason = "the skip reason's type is private to the brain, so its method can't be named"
)]
fn explain_brain(entity: Entity, brain: &Brain, score_trace: &ScoreTrace) -> Value {
    let mut behaviors = Vec::new();
    let behavior_explanations = brain
        .definition()
        .behaviors
        .iter()
        .zip(&score_trace.behavior_explanations);

    for (behavior, explanation) in behavior_explanations {
        let mut considerations = Vec::new();
        for consideration in &explanation.considerations {
            considerations.push(json!({
                "name": consideration.name,
                "input": debug_name(consideration.input),
                "raw": consideration.raw,
                "score": consideration.score,
            }));
        }

        behaviors.push(json!({
            "motion": behavior.motion.map(Motion::name),
            "action": behavior.action.map(debug_name),
            "skipped": explanation.skipped.map(|reason| reason.label()),
            "score": explanation.score,
            "momentum": explanation.momentum,
            "considerations": considerations,
        }));
    }

    json!({
        "entity": entity_id(entity),
        "tracing_started": false,
        "decided_at": score_trace.decided_at,
        "motion": brain.motion().map(Motion::name),
        "action": brain.action().map(debug_name),
        "behaviors": behaviors,
    })
}
