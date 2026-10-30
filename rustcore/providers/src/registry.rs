use std::collections::HashMap;
use std::sync::Arc;

use futures::future::join_all;

use domain::{MusicProvider, ProviderError, SearchResults};

/// Aggregates user-configured providers and fans out searches across
/// them, merging results.
#[derive(Default, Clone)]
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn MusicProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, provider: Arc<dyn MusicProvider>) {
        self.providers.insert(provider.id().to_string(), provider);
    }

    pub fn provider_ids(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.providers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }

    pub async fn search_all(&self, query: &str, limit: u32) -> Vec<SearchResults> {
        let futures = self
            .providers
            .values()
            .map(|p| {
                let query = query.to_string();
                let p = Arc::clone(p);
                async move {
                    p.search(&query, limit).await.unwrap_or_else(|e| {
                        tracing::warn!(provider = p.id(), error = %e, "search failed");
                        SearchResults::empty(p.id(), &query)
                    })
                }
            })
            .collect::<Vec<_>>();
        join_all(futures).await
    }

    pub async fn resolve_stream(
        &self,
        track: &domain::Track,
    ) -> Result<domain::StreamSource, ProviderError> {
        let provider = self.providers.get(&track.source).ok_or_else(|| {
            ProviderError::Config(format!("provider {} is not configured", track.source))
        })?;
        provider.resolve_stream(track).await
    }
}
