use std::collections::HashMap;
use std::sync::Arc;

use ind_domain::TtsProvider;

use super::adapter::TtsAdapter;

/// Resolves a `TtsProvider` variant to the concrete adapter implementation.
///
/// The registry is built once at process startup and cloned into every request
/// scope so the application layer can dispatch to the right adapter without
/// holding a lock. Each provider has at most one adapter at a time; replacing
/// an entry during tests simply overwrites it.
#[derive(Default, Clone)]
pub struct TtsAdapterRegistry {
    adapters: HashMap<TtsProvider, Arc<dyn TtsAdapter>>,
}

impl TtsAdapterRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, adapter: Arc<dyn TtsAdapter>) -> Self {
        self.adapters.insert(adapter.provider(), adapter);
        self
    }

    pub fn register(&mut self, adapter: Arc<dyn TtsAdapter>) {
        self.adapters.insert(adapter.provider(), adapter);
    }

    pub fn get(&self, provider: TtsProvider) -> Option<Arc<dyn TtsAdapter>> {
        self.adapters.get(&provider).cloned()
    }

    pub fn providers(&self) -> Vec<TtsProvider> {
        self.adapters.keys().copied().collect()
    }
}

impl std::fmt::Debug for TtsAdapterRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TtsAdapterRegistry")
            .field("providers", &self.providers())
            .finish()
    }
}
