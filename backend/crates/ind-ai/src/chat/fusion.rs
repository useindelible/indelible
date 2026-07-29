use std::cmp::Ordering;
use std::collections::HashMap;

use uuid::Uuid;

use ind_domain::SearchHit;

pub(super) fn collect_source_chunk_ids(hits: &[SearchHit]) -> Vec<Uuid> {
    hits.iter()
        .filter_map(|hit| hit.source_chunk_id.map(|id| id.into_uuid()))
        .collect()
}

/// Number of distinct S-labels offered to the model; passages share a label per document
/// (see `passages_from_hits_with_parent_context`), so this is the distinct document count.
pub(super) fn distinct_source_label_count(hits: &[SearchHit]) -> usize {
    hits.iter()
        .filter_map(|hit| hit.document_id)
        .collect::<std::collections::HashSet<_>>()
        .len()
}

pub(super) fn apply_diversity_cap(
    hits: Vec<SearchHit>,
    max_per_item: usize,
    limit: usize,
) -> Vec<SearchHit> {
    // Key by the durable search result id so multiple chunks from the same document group
    // correctly while feed preview hits remain independently addressable by delivery id.
    let mut per_subject = HashMap::<Uuid, usize>::new();
    let mut selected = Vec::new();

    for hit in hits {
        let count = per_subject.entry(hit.result_id_uuid()).or_insert(0);
        if *count >= max_per_item {
            continue;
        }
        *count += 1;
        selected.push(hit);
        if selected.len() >= limit {
            break;
        }
    }

    selected
}

pub(super) const RRF_K: f64 = 60.0;
pub(super) const MAX_PER_SECTION: usize = 2;

fn normalized_section_key(hit: &SearchHit) -> String {
    hit.section
        .as_ref()
        .map(|s| s.key.clone())
        .unwrap_or_default()
}

/// Fuse vector and FTS ranked lists using Reciprocal Rank Fusion.
///
/// Semantic and lexical hits for the same durable chunk contribute to one candidate. Coarse
/// document fallback hits have no chunk id and remain independent from semantic chunks.
/// A per-section cap prevents one section from dominating results.
pub(super) fn reciprocal_rank_fusion(
    vector_hits: Vec<SearchHit>,
    fts_hits: Vec<SearchHit>,
    max_per_section: usize,
) -> Vec<SearchHit> {
    struct Candidate {
        hit: SearchHit,
        rrf_score: f64,
        best_rank: usize,
        source_preference: u8,
    }

    let mut chunk_candidates = HashMap::<Uuid, usize>::new();
    let mut candidates: Vec<Candidate> = Vec::new();

    for (rank, hit) in vector_hits.into_iter().enumerate() {
        let idx = candidates.len();
        let rrf_contribution = 1.0 / (RRF_K + (rank as f64) + 1.0);
        if let Some(chunk_id) = hit.source_chunk_id {
            chunk_candidates.entry(chunk_id.into_uuid()).or_insert(idx);
        }
        candidates.push(Candidate {
            hit,
            rrf_score: rrf_contribution,
            best_rank: rank,
            source_preference: 0,
        });
    }

    for (rank, hit) in fts_hits.into_iter().enumerate() {
        let rrf_contribution = 1.0 / (RRF_K + (rank as f64) + 1.0);

        if let Some(candidate_idx) = hit
            .source_chunk_id
            .and_then(|chunk_id| chunk_candidates.get(&chunk_id.into_uuid()).copied())
        {
            candidates[candidate_idx].rrf_score += rrf_contribution;
            if rank < candidates[candidate_idx].best_rank {
                candidates[candidate_idx].best_rank = rank;
            }
        } else {
            let idx = candidates.len();
            if let Some(chunk_id) = hit.source_chunk_id {
                chunk_candidates.entry(chunk_id.into_uuid()).or_insert(idx);
            }
            candidates.push(Candidate {
                hit,
                rrf_score: rrf_contribution,
                best_rank: rank,
                source_preference: 1,
            });
        }
    }

    candidates.sort_by(|a, b| {
        b.rrf_score
            .partial_cmp(&a.rrf_score)
            .unwrap_or(Ordering::Equal)
            .then(a.best_rank.cmp(&b.best_rank))
            .then(a.source_preference.cmp(&b.source_preference))
            .then(b.hit.saved_at.cmp(&a.hit.saved_at))
            .then(
                b.hit
                    .result_id_uuid()
                    .to_string()
                    .cmp(&a.hit.result_id_uuid().to_string()),
            )
            .then({
                let a_key = normalized_section_key(&a.hit);
                let b_key = normalized_section_key(&b.hit);
                a_key.cmp(&b_key)
            })
            .then({
                let a_id = a.hit.source_chunk_id.map(|id| id.to_string());
                let b_id = b.hit.source_chunk_id.map(|id| id.to_string());
                a_id.cmp(&b_id)
            })
    });

    // Apply per-section cap (skip root/empty section keys — those are the common
    // document shape for HTML and PDF where all chunks share one section)
    let mut section_counts: HashMap<(Uuid, String), usize> = HashMap::new();
    let mut result = Vec::new();

    for candidate in candidates {
        let section_key = normalized_section_key(&candidate.hit);
        let is_root_section = section_key.is_empty();
        if !is_root_section {
            let count = section_counts
                .entry((candidate.hit.result_id_uuid(), section_key))
                .or_insert(0);
            if *count >= max_per_section {
                continue;
            }
            *count += 1;
        }
        let mut hit = candidate.hit;
        hit.score = candidate.rrf_score;
        result.push(hit);
    }

    result
}
