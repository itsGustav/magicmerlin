use std::collections::HashSet;

use super::{ChatType, InboundMessage};

/// Direct-message policy behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmPolicy {
    /// Any sender can direct-message the bot.
    Open,
    /// Sender must be paired first.
    Pairing,
    /// Sender must exist in allowlist.
    Allowlist,
}

/// DM policy evaluator.
#[derive(Debug, Clone)]
pub struct DmPolicyEnforcer {
    mode: DmPolicy,
    allowlist: HashSet<String>,
    paired_users: HashSet<String>,
}

impl DmPolicyEnforcer {
    /// Creates a new enforcer in the selected policy mode.
    pub fn new(mode: DmPolicy) -> Self {
        Self {
            mode,
            allowlist: HashSet::new(),
            paired_users: HashSet::new(),
        }
    }

    /// Adds a sender id to allowlist.
    pub fn allow_user(&mut self, user_id: impl Into<String>) {
        self.allowlist.insert(user_id.into());
    }

    /// Approves a sender id for pairing mode.
    pub fn approve_pairing(&mut self, user_id: impl Into<String>) {
        self.paired_users.insert(user_id.into());
    }

    /// Returns whether an inbound message passes DM policy checks.
    pub fn allows(&self, message: &InboundMessage) -> bool {
        if message.chat_type != ChatType::Direct {
            return true;
        }

        match self.mode {
            DmPolicy::Open => true,
            DmPolicy::Pairing => self.paired_users.contains(&message.sender.id),
            DmPolicy::Allowlist => self.allowlist.contains(&message.sender.id),
        }
    }
}

/// Group mention gating rules.
#[derive(Debug, Clone)]
pub struct MentionGate {
    bot_name: String,
    require_mention_in_groups: bool,
}

impl MentionGate {
    /// Creates a mention gate configuration for the active bot handle.
    pub fn new(bot_name: impl Into<String>, require_mention_in_groups: bool) -> Self {
        Self {
            bot_name: bot_name.into(),
            require_mention_in_groups,
        }
    }

    /// Returns whether an inbound message should be processed.
    pub fn should_process(&self, message: &InboundMessage) -> bool {
        if message.chat_type == ChatType::Direct || !self.require_mention_in_groups {
            return true;
        }

        let Some(text) = message.text.as_deref() else {
            return false;
        };

        let mention = format!("@{}", self.bot_name);
        text.split_whitespace().any(|part| part == mention)
    }
}
