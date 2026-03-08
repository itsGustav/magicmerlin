//! Model registry with alias resolution, recommendations, and metadata-driven cost calculation.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{ProviderError, Result};
use crate::model_catalog::built_in_models;

/// Model capability flags.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelCapabilities {
    /// Whether the model accepts images.
    pub vision: bool,
    /// Whether the model supports tools.
    pub tools: bool,
    /// Whether the model supports streaming.
    pub streaming: bool,
    /// Whether model supports JSON mode.
    pub json_mode: bool,
    /// Whether model supports reasoning controls.
    pub reasoning: bool,
}

/// Model metadata entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDefinition {
    /// Provider name.
    pub provider: String,
    /// Provider-local model id.
    pub model_id: String,
    /// Maximum context tokens.
    pub context_window: u32,
    /// Max output tokens.
    pub max_tokens: u32,
    /// Input cost USD per 1M tokens.
    pub input_cost_per_mtok: f64,
    /// Output cost USD per 1M tokens.
    pub output_cost_per_mtok: f64,
    /// Capability flags.
    pub capabilities: ModelCapabilities,
}

/// Requirements for model recommendation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelRequirements {
    /// Minimum context window.
    pub min_context_window: u32,
    /// Whether vision support is required.
    pub require_vision: bool,
    /// Whether tool support is required.
    pub require_tools: bool,
    /// Whether streaming support is required.
    pub require_streaming: bool,
    /// Whether JSON mode support is required.
    pub require_json_mode: bool,
    /// Whether reasoning support is required.
    pub require_reasoning: bool,
    /// Preferred providers in rank order.
    #[serde(default)]
    pub preferred_providers: Vec<String>,
    /// Excluded providers.
    #[serde(default)]
    pub excluded_providers: Vec<String>,
}

/// Runtime model registry and aliases.
#[derive(Debug, Clone, Default)]
pub struct ModelRegistry {
    models: HashMap<String, ModelDefinition>,
    aliases: HashMap<String, String>,
}

impl ModelRegistry {
    /// Builds a model registry from config and default aliases.
    pub fn from_config(config: &magicmerlin_config::Config) -> Result<Self> {
        let mut this = Self::default();
        this.seed_built_ins();
        this.seed_default_aliases();

        let providers = config
            .models
            .values
            .get("providers")
            .and_then(Value::as_object);

        if let Some(providers_obj) = providers {
            for (provider_name, provider_value) in providers_obj {
                parse_provider_block(&mut this, provider_name, provider_value)?;
            }
        }

        Ok(this)
    }

    /// Seeds the built-in model database.
    pub fn seed_built_ins(&mut self) {
        for model in built_in_models() {
            self.upsert_model(model);
        }
    }

    /// Inserts or replaces one model definition.
    pub fn upsert_model(&mut self, def: ModelDefinition) {
        let key = format!("{}/{}", def.provider, def.model_id);
        self.models.insert(key, def);
    }

    /// Inserts one alias mapping.
    pub fn upsert_alias(&mut self, alias: impl Into<String>, model: impl Into<String>) {
        self.aliases.insert(alias.into(), model.into());
    }

    /// Resolves model alias or canonical identifier.
    pub fn resolve_model(&self, model_or_alias: &str) -> Result<String> {
        if let Some(canonical) = self.aliases.get(model_or_alias) {
            return Ok(canonical.clone());
        }

        if model_or_alias.contains('/') {
            if self.models.contains_key(model_or_alias) {
                return Ok(model_or_alias.to_string());
            }
            let (provider, model_id) = Self::parse_provider_model(model_or_alias)?;
            if self
                .models
                .contains_key(format!("{provider}/{model_id}").as_str())
            {
                return Ok(model_or_alias.to_string());
            }
        }

        Err(ProviderError::Model(format!(
            "unknown model alias: {model_or_alias}"
        )))
    }

    /// Splits canonical `provider/model-id` model string.
    pub fn parse_provider_model(model: &str) -> Result<(String, String)> {
        let Some((provider, model_id)) = model.split_once('/') else {
            return Err(ProviderError::Model(format!(
                "model must use provider/model format: {model}"
            )));
        };
        if provider.is_empty() || model_id.is_empty() {
            return Err(ProviderError::Model(format!("invalid model: {model}")));
        }
        Ok((provider.to_string(), model_id.to_string()))
    }

    /// Returns model metadata for canonical id.
    pub fn model(&self, canonical: &str) -> Option<&ModelDefinition> {
        self.models.get(canonical)
    }

    /// Returns all canonical model IDs.
    pub fn all_models(&self) -> Vec<String> {
        let mut models = self.models.keys().cloned().collect::<Vec<_>>();
        models.sort();
        models
    }

    /// Returns all aliases.
    pub fn aliases(&self) -> &HashMap<String, String> {
        &self.aliases
    }

    /// Estimates request cost from usage counters.
    pub fn estimate_cost_usd(&self, canonical: &str, usage: &crate::types::Usage) -> Option<f64> {
        let model = self.models.get(canonical)?;
        let in_cost = (usage.input_tokens as f64 / 1_000_000.0) * model.input_cost_per_mtok;
        let out_cost = (usage.output_tokens as f64 / 1_000_000.0) * model.output_cost_per_mtok;
        Some(in_cost + out_cost)
    }

    /// Recommends a model from requirements.
    pub fn recommend_model(&self, requirements: &ModelRequirements) -> Option<String> {
        let excluded = requirements
            .excluded_providers
            .iter()
            .map(|v| v.to_ascii_lowercase())
            .collect::<HashSet<_>>();
        let preferred = requirements
            .preferred_providers
            .iter()
            .map(|v| v.to_ascii_lowercase())
            .collect::<Vec<_>>();

        let mut ranked = self
            .models
            .iter()
            .filter(|(_, model)| model.context_window >= requirements.min_context_window)
            .filter(|(_, model)| !requirements.require_vision || model.capabilities.vision)
            .filter(|(_, model)| !requirements.require_tools || model.capabilities.tools)
            .filter(|(_, model)| !requirements.require_streaming || model.capabilities.streaming)
            .filter(|(_, model)| !requirements.require_json_mode || model.capabilities.json_mode)
            .filter(|(_, model)| !requirements.require_reasoning || model.capabilities.reasoning)
            .filter(|(_, model)| !excluded.contains(&model.provider.to_ascii_lowercase()))
            .map(|(canonical, model)| {
                let provider_rank = preferred
                    .iter()
                    .position(|provider| provider == &model.provider.to_ascii_lowercase())
                    .unwrap_or(usize::MAX);
                let cost_score = model.input_cost_per_mtok + model.output_cost_per_mtok;
                (
                    canonical.clone(),
                    provider_rank,
                    cost_score,
                    u64::from(model.context_window),
                )
            })
            .collect::<Vec<_>>();

        ranked.sort_by(|a, b| {
            a.1.cmp(&b.1)
                .then_with(|| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
                .then_with(|| b.3.cmp(&a.3))
        });

        ranked.first().map(|entry| entry.0.clone())
    }

    /// Returns provider -> models mapping.
    pub fn models_by_provider(&self) -> BTreeMap<String, Vec<String>> {
        let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for canonical in self.models.keys() {
            if let Some((provider, _)) = canonical.split_once('/') {
                map.entry(provider.to_string())
                    .or_default()
                    .push(canonical.clone());
            }
        }
        for models in map.values_mut() {
            models.sort();
        }
        map
    }

    /// Seeds required built-in aliases.
    pub fn seed_default_aliases(&mut self) {
        self.aliases
            .insert("gpt".to_string(), "openai/gpt-5.2".to_string());
        self.aliases.insert(
            "sonnet".to_string(),
            "anthropic/claude-sonnet-4-6".to_string(),
        );
        self.aliases
            .insert("opus".to_string(), "anthropic/claude-opus-4-6".to_string());
        self.aliases
            .insert("gemini".to_string(), "google/gemini-2.5-pro".to_string());
        self.aliases
            .insert("fast".to_string(), "openai/gpt-5.2-mini".to_string());
    }
}

fn parse_provider_block(
    registry: &mut ModelRegistry,
    provider_name: &str,
    value: &Value,
) -> Result<()> {
    let models = value
        .get("models")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            ProviderError::Model(format!("missing models array for provider {provider_name}"))
        })?;

    for entry in models {
        let model_id = entry.get("id").and_then(Value::as_str).ok_or_else(|| {
            ProviderError::Model(format!("missing id for provider {provider_name}"))
        })?;

        let def = ModelDefinition {
            provider: provider_name.to_string(),
            model_id: model_id.to_string(),
            context_window: as_u32(entry.get("context_window"), 128_000),
            max_tokens: as_u32(entry.get("max_tokens"), 8_192),
            input_cost_per_mtok: as_f64(entry.get("input_cost_per_mtok"), 0.0),
            output_cost_per_mtok: as_f64(entry.get("output_cost_per_mtok"), 0.0),
            capabilities: ModelCapabilities {
                vision: as_bool(entry.get("vision"), false),
                tools: as_bool(entry.get("tools"), true),
                streaming: as_bool(entry.get("streaming"), true),
                json_mode: as_bool(entry.get("json_mode"), false),
                reasoning: as_bool(entry.get("reasoning"), false),
            },
        };
        registry.upsert_model(def);

        if let Some(aliases) = entry.get("aliases").and_then(Value::as_array) {
            for alias in aliases.iter().filter_map(Value::as_str) {
                registry.upsert_alias(alias, format!("{provider_name}/{model_id}"));
            }
        }
    }

    Ok(())
}

fn as_u32(v: Option<&Value>, default: u32) -> u32 {
    v.and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        .unwrap_or(default)
}

fn as_f64(v: Option<&Value>, default: f64) -> f64 {
    v.and_then(Value::as_f64).unwrap_or(default)
}

fn as_bool(v: Option<&Value>, default: bool) -> bool {
    v.and_then(Value::as_bool).unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_default_aliases() {
        let config = magicmerlin_config::Config::default();
        let registry = ModelRegistry::from_config(&config).expect("registry");
        assert_eq!(
            registry.resolve_model("gpt").expect("alias"),
            "openai/gpt-5.2"
        );
    }

    #[test]
    fn estimates_cost() {
        let mut registry = ModelRegistry::default();
        registry.upsert_model(ModelDefinition {
            provider: "openai".to_string(),
            model_id: "gpt-5.2".to_string(),
            context_window: 1,
            max_tokens: 1,
            input_cost_per_mtok: 2.0,
            output_cost_per_mtok: 4.0,
            capabilities: ModelCapabilities::default(),
        });
        let usage = crate::types::Usage {
            input_tokens: 1_000_000,
            output_tokens: 500_000,
            cache_read: 0,
            cache_write: 0,
        };
        let cost = registry
            .estimate_cost_usd("openai/gpt-5.2", &usage)
            .expect("cost");
        assert!((cost - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn recommendation_prefers_lower_cost_with_capabilities() {
        let mut registry = ModelRegistry::default();
        registry.upsert_model(ModelDefinition {
            provider: "openai".to_string(),
            model_id: "gpt-5.2".to_string(),
            context_window: 128_000,
            max_tokens: 16_384,
            input_cost_per_mtok: 5.0,
            output_cost_per_mtok: 15.0,
            capabilities: ModelCapabilities {
                vision: true,
                tools: true,
                streaming: true,
                json_mode: true,
                reasoning: true,
            },
        });
        registry.upsert_model(ModelDefinition {
            provider: "google".to_string(),
            model_id: "gemini-2.5-pro".to_string(),
            context_window: 1_000_000,
            max_tokens: 65_000,
            input_cost_per_mtok: 3.0,
            output_cost_per_mtok: 10.0,
            capabilities: ModelCapabilities {
                vision: true,
                tools: true,
                streaming: true,
                json_mode: true,
                reasoning: true,
            },
        });

        let recommended = registry
            .recommend_model(&ModelRequirements {
                min_context_window: 64_000,
                require_vision: true,
                require_tools: true,
                require_streaming: true,
                require_json_mode: true,
                require_reasoning: true,
                preferred_providers: Vec::new(),
                excluded_providers: Vec::new(),
            })
            .expect("recommended");

        assert_eq!(recommended, "google/gemini-2.5-pro");
    }

    #[test]
    fn built_ins_seed_non_empty_registry() {
        let mut registry = ModelRegistry::default();
        registry.seed_built_ins();
        assert!(registry.all_models().len() > 20);
    }
}
