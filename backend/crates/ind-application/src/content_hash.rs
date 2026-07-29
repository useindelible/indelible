/// SimHash over 3-gram token shingles of whitespace-normalised input.
/// Returns a 16-character lowercase hex string (64-bit fingerprint).
/// Used for near-duplicate detection: two items with Hamming distance ≤ 3
/// are considered near-duplicates.
pub fn compute_content_hash(content: &str) -> String {
    let tokens = tokenize(content);
    if tokens.is_empty() {
        return format!("{:016x}", 0u64);
    }

    let shingles = trigram_shingles(&tokens);

    let mut tally: [i64; 64] = [0; 64];
    for shingle in &shingles {
        let h = fnv1a_64(shingle.as_bytes());
        for bit in 0u64..64 {
            if (h >> bit) & 1 == 1 {
                tally[bit as usize] += 1;
            } else {
                tally[bit as usize] -= 1;
            }
        }
    }

    let mut fingerprint: u64 = 0;
    for bit in 0u64..64 {
        if tally[bit as usize] > 0 {
            fingerprint |= 1 << bit;
        }
    }

    format!("{:016x}", fingerprint)
}

fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|w| w.to_lowercase())
        .filter(|w| !w.is_empty())
        .collect()
}

fn trigram_shingles(tokens: &[String]) -> Vec<String> {
    if tokens.len() < 3 {
        return tokens.to_vec();
    }
    tokens
        .windows(3)
        .map(|w| format!("{} {} {}", w[0], w[1], w[2]))
        .collect()
}

// FNV-1a 64-bit — no external dependency, well-distributed for short strings.
fn fnv1a_64(data: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
    const FNV_PRIME: u64 = 1_099_511_628_211;
    let mut hash = FNV_OFFSET;
    for &byte in data {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
