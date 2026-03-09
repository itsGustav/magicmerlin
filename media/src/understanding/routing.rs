use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use super::{MediaType, UnderstandingClient, VisionProvider};
use crate::{MediaError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProviderCost {
    pub input_per_million: f32,
    pub output_per_million: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderDecision {
    pub provider: VisionProvider,
    pub reason: String,
    pub estimated_cost_rank: usize,
    pub supports_media: bool,
}

impl UnderstandingClient {
    pub fn estimate_provider_cost(&self, provider: VisionProvider) -> ProviderCost {
        match provider {
            VisionProvider::OpenAi => ProviderCost {
                input_per_million: 5.0,
                output_per_million: 15.0,
            },
            VisionProvider::Anthropic => ProviderCost {
                input_per_million: 3.0,
                output_per_million: 15.0,
            },
            VisionProvider::Google => ProviderCost {
                input_per_million: 0.3,
                output_per_million: 1.25,
            },
            VisionProvider::Local => ProviderCost {
                input_per_million: 0.01,
                output_per_million: 0.01,
            },
        }
    }

    pub fn route_provider_by_cost(
        &self,
        media_type: MediaType,
        preferred: Option<VisionProvider>,
    ) -> Result<ProviderDecision> {
        if let Some(provider) = preferred {
            if self.provider_available(provider) {
                return Ok(ProviderDecision {
                    provider,
                    reason: "preferred provider requested and configured".to_string(),
                    estimated_cost_rank: 1,
                    supports_media: self
                        .provider_capabilities()
                        .get(&provider)
                        .map(|v| v.contains(&media_type))
                        .unwrap_or(false),
                });
            }
            return Err(MediaError::InvalidInput(format!(
                "preferred provider {:?} is not available",
                provider
            )));
        }

        let capabilities = self.provider_capabilities();
        let mut candidates = vec![
            VisionProvider::OpenAi,
            VisionProvider::Anthropic,
            VisionProvider::Google,
            VisionProvider::Local,
        ];
        candidates.retain(|provider| {
            self.provider_available(*provider)
                && capabilities
                    .get(provider)
                    .map(|types| types.contains(&media_type))
                    .unwrap_or(false)
        });

        if candidates.is_empty() {
            return Err(MediaError::InvalidInput(
                "no provider supports requested media type".to_string(),
            ));
        }

        candidates.sort_by(|a, b| {
            let ac = self.estimate_provider_cost(*a);
            let bc = self.estimate_provider_cost(*b);
            let at = ac.input_per_million + ac.output_per_million;
            let bt = bc.input_per_million + bc.output_per_million;
            at.partial_cmp(&bt).unwrap_or(Ordering::Equal)
        });

        let picked = candidates[0];
        let rank = candidates
            .iter()
            .position(|p| *p == picked)
            .map(|p| p + 1)
            .unwrap_or(1);

        Ok(ProviderDecision {
            provider: picked,
            reason: format!("lowest estimated cost for {:?}", media_type),
            estimated_cost_rank: rank,
            supports_media: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client() -> UnderstandingClient {
        let mut cfg = super::super::UnderstandingConfig::default();
        cfg.openai_api_key = Some("x".to_string());
        cfg.google_api_key = Some("y".to_string());
        UnderstandingClient::new(cfg)
    }

    #[test]
    fn cost_router_prefers_low_cost_provider() {
        let client = client();
        let decision = client
            .route_provider_by_cost(MediaType::Image, None)
            .expect("decision");
        assert!(matches!(decision.provider, VisionProvider::Google));
    }
}
