//! Built-in model catalog used by `ModelRegistry`.

use crate::model_registry::{ModelCapabilities, ModelDefinition};

fn model(
    provider: &str,
    model_id: &str,
    context_window: u32,
    max_tokens: u32,
    input_cost_per_mtok: f64,
    output_cost_per_mtok: f64,
    capabilities: ModelCapabilities,
) -> ModelDefinition {
    ModelDefinition {
        provider: provider.to_string(),
        model_id: model_id.to_string(),
        context_window,
        max_tokens,
        input_cost_per_mtok,
        output_cost_per_mtok,
        capabilities,
    }
}

/// Returns built-in model definitions across first-party and compatible providers.
pub fn built_in_models() -> Vec<ModelDefinition> {
    let mut out = Vec::new();
    out.push(model(
        "openai",
        "gpt-5.2",
        400000,
        128000,
        2.00,
        8.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "openai",
        "gpt-5.2-mini",
        400000,
        64000,
        0.80,
        3.20,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "openai",
        "gpt-5.2-nano",
        200000,
        32000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "o3",
        200000,
        100000,
        4.00,
        16.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "openai",
        "o3-mini",
        200000,
        65536,
        1.20,
        4.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "openai",
        "o4-mini",
        256000,
        65536,
        1.50,
        6.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "openai",
        "chatgpt-4o-latest",
        128000,
        16384,
        5.00,
        20.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-1",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-2",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-3",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-4",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-5",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-6",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-7",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-8",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-9",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-10",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-11",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-12",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-13",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-14",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-15",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-16",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-17",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-18",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-19",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-20",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-21",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-22",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-23",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-24",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-25",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-26",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-27",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-28",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-29",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-30",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-31",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-32",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-33",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-34",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-35",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-36",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-37",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-38",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-39",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-40",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-41",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-42",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-43",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-44",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "openai",
        "gpt-4o-archive-45",
        128000,
        16384,
        3.00,
        12.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-opus-4-6",
        200000,
        32000,
        15.00,
        75.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-4-6",
        200000,
        64000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-haiku-4-5",
        200000,
        32000,
        0.80,
        4.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-1",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-2",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-3",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-4",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-5",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-6",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-7",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-8",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-9",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-10",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-11",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-12",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-13",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-14",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-15",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-16",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-17",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-18",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-19",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-20",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-21",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-22",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-23",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-24",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-25",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-26",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-27",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-28",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-29",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-30",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-31",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-32",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-33",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-34",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-35",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-36",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-37",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-38",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-39",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-40",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-41",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-42",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-43",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-44",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-45",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-46",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "anthropic",
        "claude-sonnet-archive-47",
        200000,
        16000,
        3.00,
        15.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "google",
        "gemini-2.5-pro",
        1000000,
        65536,
        1.25,
        5.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "google",
        "gemini-2.5-flash",
        1000000,
        65536,
        0.35,
        1.20,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "google",
        "gemini-2.0-flash",
        1000000,
        32768,
        0.20,
        0.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-1",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-2",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-3",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-4",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-5",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-6",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-7",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-8",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-9",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-10",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-11",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-12",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-13",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-14",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-15",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-16",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-17",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-18",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-19",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-20",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-21",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-22",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-23",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-24",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-25",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-26",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-27",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-28",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-29",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-30",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-31",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-32",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-33",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-34",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-35",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-36",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-37",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-38",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-39",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-40",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-41",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-42",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-43",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-44",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "google",
        "gemini-experimental-45",
        512000,
        32768,
        0.45,
        1.80,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-4",
        256000,
        64000,
        5.00,
        20.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "xai",
        "grok-3-mini",
        256000,
        32000,
        1.50,
        6.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-1",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-2",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-3",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-4",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-5",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-6",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-7",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-8",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-9",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-10",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-11",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-12",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-13",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-14",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-15",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-16",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-17",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-18",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-19",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-20",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-21",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-22",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-23",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-24",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-25",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-26",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-27",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-28",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-29",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-30",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-31",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-32",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-33",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-34",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-35",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-36",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-37",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "xai",
        "grok-legacy-38",
        128000,
        16384,
        2.50,
        10.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "llama-4-scout",
        128000,
        16384,
        0.10,
        0.40,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "llama-4-maverick",
        128000,
        16384,
        0.20,
        0.80,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "qwen-qwq-32b",
        128000,
        16384,
        0.35,
        1.40,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-1",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-2",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-3",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-4",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-5",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-6",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-7",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-8",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-9",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-10",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-11",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-12",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-13",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-14",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-15",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-16",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-17",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-18",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-19",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-20",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-21",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-22",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-23",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-24",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-25",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-26",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-27",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-28",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-29",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-30",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-31",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-32",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-33",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-34",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-35",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-36",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-37",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-38",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-39",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-40",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-41",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-42",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-43",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-44",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "groq",
        "ultra-fast-45",
        64000,
        8192,
        0.08,
        0.32,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-large-2.1",
        128000,
        32000,
        2.00,
        6.00,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-medium-3",
        64000,
        16000,
        0.70,
        2.10,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-small-3.1",
        64000,
        16000,
        0.20,
        0.60,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-1",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-2",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-3",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-4",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-5",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-6",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-7",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-8",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-9",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-10",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-11",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-12",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-13",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-14",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-15",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-16",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-17",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-18",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-19",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-20",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-21",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-22",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-23",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-24",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-25",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-26",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-27",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-28",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-29",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-30",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-31",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-32",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-33",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-34",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "mistral",
        "mistral-archive-35",
        32000,
        8192,
        0.30,
        0.90,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-reasoner",
        128000,
        32000,
        0.55,
        2.20,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-chat",
        128000,
        16000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-1",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-2",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-3",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-4",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-5",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-6",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-7",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-8",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-9",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-10",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-11",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-12",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-13",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-14",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-15",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-16",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-17",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-18",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-19",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-20",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-21",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-22",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-23",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-24",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-25",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-26",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-27",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "deepseek",
        "deepseek-legacy-28",
        64000,
        8192,
        0.18,
        0.72,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-k2",
        200000,
        32000,
        0.70,
        2.50,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: true,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-1",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-2",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-3",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-4",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-5",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-6",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-7",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-8",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-9",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-10",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-11",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-12",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-13",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-14",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-15",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-16",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-17",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-18",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-19",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-20",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-21",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-22",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-23",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-24",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "moonshot",
        "kimi-archive-25",
        128000,
        12000,
        0.40,
        1.40,
        ModelCapabilities {
            vision: true,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-text-01",
        128000,
        24000,
        0.30,
        1.20,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-1",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-2",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-3",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-4",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-5",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-6",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-7",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-8",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-9",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-10",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-11",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-12",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-13",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-14",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-15",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-16",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-17",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-18",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-19",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-20",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-21",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-22",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-23",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-24",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "minimax",
        "minimax-chat-25",
        64000,
        12000,
        0.25,
        1.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "llama3.3-70b",
        128000,
        16000,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "qwen2.5-coder-32b",
        128000,
        16000,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: true,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-1",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-2",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-3",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-4",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-5",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-6",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-7",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-8",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-9",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-10",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-11",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-12",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-13",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-14",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-15",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-16",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-17",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-18",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-19",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-20",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-21",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-22",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-23",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-24",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-25",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-26",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-27",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-28",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-29",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-30",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-31",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-32",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-33",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-34",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out.push(model(
        "local",
        "local-model-35",
        32000,
        8192,
        0.00,
        0.00,
        ModelCapabilities {
            vision: false,
            tools: true,
            streaming: true,
            json_mode: false,
            reasoning: false,
        },
    ));
    out
}
