use magicmerlin_auto_reply::{AutoReplyEngine, InboundMessage as PipelineInboundMessage, PipelineDecision};

use super::{ChatType, InboundMessage};

/// Bridge from channel normalization into the auto-reply pipeline.
#[derive(Debug)]
pub struct AutoReplyBridge {
    engine: AutoReplyEngine,
}

impl AutoReplyBridge {
    /// Creates a new bridge around an existing auto-reply engine.
    pub fn new(engine: AutoReplyEngine) -> Self {
        Self { engine }
    }

    /// Maps normalized channel message into auto-reply input and evaluates policy.
    pub fn evaluate(&mut self, inbound: &InboundMessage, bot_name: &str) -> PipelineDecision {
        self.engine
            .evaluate_inbound(&to_pipeline_message(inbound, bot_name))
    }

    /// Returns immutable reference to inner engine.
    pub fn engine(&self) -> &AutoReplyEngine {
        &self.engine
    }

    /// Returns mutable reference to inner engine.
    pub fn engine_mut(&mut self) -> &mut AutoReplyEngine {
        &mut self.engine
    }
}

fn to_pipeline_message(inbound: &InboundMessage, bot_name: &str) -> PipelineInboundMessage {
    let text = inbound.text.clone().unwrap_or_default();
    let mention = format!("@{bot_name}");
    let mentioned = text.split_whitespace().any(|part| part == mention);

    PipelineInboundMessage {
        channel: format!("{:?}", inbound.platform).to_lowercase(),
        chat_id: Some(inbound.chat_id.clone()),
        user_id: inbound.sender.id.clone(),
        text,
        is_dm: inbound.chat_type == ChatType::Direct,
        mentioned,
        priority: 1,
    }
}
