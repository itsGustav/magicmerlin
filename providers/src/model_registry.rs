//! Model registry with alias resolution and metadata-driven cost calculation.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{ProviderError, Result};

/// Model capability flags.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelCapabilities {
    /// Whether the model accepts images.
    pub vision: bool,
    /// Whether the model supports tools.
    pub tools: bool,
    /// Whether the model supports streaming.
    pub streaming: bool,
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
            return Ok(model_or_alias.to_string());
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

    /// Estimates request cost from usage counters.
    pub fn estimate_cost_usd(&self, canonical: &str, usage: &crate::types::Usage) -> Option<f64> {
        let model = self.models.get(canonical)?;
        let in_cost = (usage.input_tokens as f64 / 1_000_000.0) * model.input_cost_per_mtok;
        let out_cost = (usage.output_tokens as f64 / 1_000_000.0) * model.output_cost_per_mtok;
        Some(in_cost + out_cost)
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
}
