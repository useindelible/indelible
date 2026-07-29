use ind_domain::{Entity, EntityType, ExtractedEntity};

use crate::untrusted::{GUIDANCE, fence};

pub(crate) const ENTITY_RESOLUTION_SYSTEM_PROMPT: &str = "You are given a document title and a \
numbered list of newly extracted entities. For EACH entity you are given its description, aliases, \
and a list of candidate existing entities of the same type (each numbered, with a description). For \
each entity, decide which ONE of ITS candidates, if any, is the SAME real-world referent. Use the \
descriptions and document context, not just the names. Be conservative: choose a candidate only if \
confident they are the exact same person, organization, place, event, or work — not merely related. \
A parent brand and its product are NOT the same (Amazon vs Amazon Bedrock => null). Things merely \
sharing letters are NOT the same (React vs ReAct => null; Swift vs SWIFT => null). An acronym and \
its expansion ARE the same (AWS vs Amazon Web Services => match). A fuller form of one person's name \
IS the same (Elon Musk vs Elon Reeve Musk => match); a bare given name (Daniel vs Daniel Abadi) is a \
match ONLY if the descriptions/document make it unambiguously the same person, otherwise null. \
Return {\"results\": [{\"entity\": <entity number>, \"match\": <candidate number or null>, \
\"confidence\": 0..1}, ...]} with exactly one object per entity. Output only the JSON object.";

/// One newly extracted entity plus the candidate existing entities surfaced for it.
pub(crate) struct AdjudicationItem<'a> {
    pub entity: &'a ExtractedEntity,
    pub candidates: &'a [Entity],
}

pub(crate) fn build_batch_resolution_prompt(doc_title: &str, items: &[AdjudicationItem]) -> String {
    let mut body = format!("Document: \"{doc_title}\"\n");
    for (index, item) in items.iter().enumerate() {
        let entity = item.entity;
        let description = entity.description.as_deref().unwrap_or("(none)");
        let aliases = if entity.aliases.is_empty() {
            "(none)".to_string()
        } else {
            entity.aliases.join(", ")
        };
        body.push_str(&format!(
            "\nEntity {number}:\n  name: {name}\n  type: {ty}\n  description: {description}\n  \
             also known as: {aliases}\n  candidates:\n",
            number = index + 1,
            name = entity.name,
            ty = entity_type_label(entity.entity_type),
        ));
        for (candidate_index, candidate) in item.candidates.iter().enumerate() {
            let candidate_description = candidate.description.as_deref().unwrap_or("(none)");
            body.push_str(&format!(
                "    {number}. name: {name}   description: {candidate_description}\n",
                number = candidate_index + 1,
                name = candidate.name,
            ));
        }
    }

    // The document title, entity names/descriptions/aliases, and candidate
    // descriptions are all model-extracted from untrusted document content, so
    // fence them and tell the model to treat them as data, not instructions.
    format!(
        "{GUIDANCE}\n\n{}\n\nFor each numbered entity, which candidate (if any) is the SAME \
         real-world referent?",
        fence(&body)
    )
}

fn entity_type_label(entity_type: EntityType) -> &'static str {
    match entity_type {
        EntityType::Person => "person",
        EntityType::Organization => "organization",
        EntityType::Location => "location",
        EntityType::Event => "event",
        EntityType::Work => "work",
    }
}
