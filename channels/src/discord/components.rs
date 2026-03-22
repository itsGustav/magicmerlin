//! Discord message components: buttons, select menus, action rows, and modals.
//!
//! Maps to Discord's Component API v10 — buttons (style 1-5), string/user/role/channel
//! select menus, text inputs, and modal submit flows.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// ---------------------------------------------------------------------------
// Component model
// ---------------------------------------------------------------------------

/// Top-level action row that can contain buttons or a single select menu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionRow {
    pub components: Vec<Component>,
}

impl ActionRow {
    pub fn buttons(buttons: Vec<Button>) -> Self {
        Self {
            components: buttons.into_iter().map(Component::Button).collect(),
        }
    }

    pub fn select_menu(menu: SelectMenu) -> Self {
        Self {
            components: vec![Component::SelectMenu(menu)],
        }
    }

    pub fn text_input(input: TextInput) -> Self {
        Self {
            components: vec![Component::TextInput(input)],
        }
    }
}

/// Individual component within an action row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Component {
    Button(Button),
    SelectMenu(SelectMenu),
    TextInput(TextInput),
}

/// Button style matching Discord API (1-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonStyle {
    /// Blurple button.
    Primary = 1,
    /// Grey button.
    Secondary = 2,
    /// Green button.
    Success = 3,
    /// Red button.
    Danger = 4,
    /// Grey button that navigates to a URL.
    Link = 5,
}

/// Interactive button component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Button {
    pub custom_id: Option<String>,
    pub label: String,
    pub style: ButtonStyle,
    pub url: Option<String>,
    pub emoji: Option<String>,
    pub disabled: bool,
}

impl Button {
    pub fn primary(custom_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            custom_id: Some(custom_id.into()),
            label: label.into(),
            style: ButtonStyle::Primary,
            url: None,
            emoji: None,
            disabled: false,
        }
    }

    pub fn secondary(custom_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            custom_id: Some(custom_id.into()),
            label: label.into(),
            style: ButtonStyle::Secondary,
            url: None,
            emoji: None,
            disabled: false,
        }
    }

    pub fn success(custom_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            custom_id: Some(custom_id.into()),
            label: label.into(),
            style: ButtonStyle::Success,
            url: None,
            emoji: None,
            disabled: false,
        }
    }

    pub fn danger(custom_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            custom_id: Some(custom_id.into()),
            label: label.into(),
            style: ButtonStyle::Danger,
            url: None,
            emoji: None,
            disabled: false,
        }
    }

    pub fn link(url: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            custom_id: None,
            label: label.into(),
            style: ButtonStyle::Link,
            url: Some(url.into()),
            emoji: None,
            disabled: false,
        }
    }

    pub fn with_emoji(mut self, emoji: impl Into<String>) -> Self {
        self.emoji = Some(emoji.into());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// Select menu kind (string, user, role, mentionable, channel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectMenuKind {
    String,
    User,
    Role,
    Mentionable,
    Channel,
}

/// Option within a string select menu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectOption {
    pub label: String,
    pub value: String,
    pub description: Option<String>,
    pub emoji: Option<String>,
    pub default: bool,
}

impl SelectOption {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            description: None,
            emoji: None,
            default: false,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn as_default(mut self) -> Self {
        self.default = true;
        self
    }
}

/// Select menu component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectMenu {
    pub custom_id: String,
    pub kind: SelectMenuKind,
    pub placeholder: Option<String>,
    pub min_values: u8,
    pub max_values: u8,
    pub options: Vec<SelectOption>,
    pub disabled: bool,
}

impl SelectMenu {
    pub fn string(custom_id: impl Into<String>, options: Vec<SelectOption>) -> Self {
        Self {
            custom_id: custom_id.into(),
            kind: SelectMenuKind::String,
            placeholder: None,
            min_values: 1,
            max_values: 1,
            options,
            disabled: false,
        }
    }

    pub fn user(custom_id: impl Into<String>) -> Self {
        Self {
            custom_id: custom_id.into(),
            kind: SelectMenuKind::User,
            placeholder: None,
            min_values: 1,
            max_values: 1,
            options: Vec::new(),
            disabled: false,
        }
    }

    pub fn role(custom_id: impl Into<String>) -> Self {
        Self {
            custom_id: custom_id.into(),
            kind: SelectMenuKind::Role,
            placeholder: None,
            min_values: 1,
            max_values: 1,
            options: Vec::new(),
            disabled: false,
        }
    }

    pub fn channel(custom_id: impl Into<String>) -> Self {
        Self {
            custom_id: custom_id.into(),
            kind: SelectMenuKind::Channel,
            placeholder: None,
            min_values: 1,
            max_values: 1,
            options: Vec::new(),
            disabled: false,
        }
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn with_range(mut self, min: u8, max: u8) -> Self {
        self.min_values = min;
        self.max_values = max;
        self
    }
}

/// Text input style for modals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextInputStyle {
    Short = 1,
    Paragraph = 2,
}

/// Text input component (only valid inside modals).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextInput {
    pub custom_id: String,
    pub label: String,
    pub style: TextInputStyle,
    pub placeholder: Option<String>,
    pub value: Option<String>,
    pub required: bool,
    pub min_length: Option<u16>,
    pub max_length: Option<u16>,
}

impl TextInput {
    pub fn short(custom_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            custom_id: custom_id.into(),
            label: label.into(),
            style: TextInputStyle::Short,
            placeholder: None,
            value: None,
            required: true,
            min_length: None,
            max_length: None,
        }
    }

    pub fn paragraph(custom_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            custom_id: custom_id.into(),
            label: label.into(),
            style: TextInputStyle::Paragraph,
            placeholder: None,
            value: None,
            required: true,
            min_length: None,
            max_length: None,
        }
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn with_length(mut self, min: u16, max: u16) -> Self {
        self.min_length = Some(min);
        self.max_length = Some(max);
        self
    }
}

/// Modal dialog containing text inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Modal {
    pub custom_id: String,
    pub title: String,
    pub components: Vec<ActionRow>,
}

impl Modal {
    pub fn new(custom_id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            custom_id: custom_id.into(),
            title: title.into(),
            components: Vec::new(),
        }
    }

    pub fn add_input(mut self, input: TextInput) -> Self {
        self.components.push(ActionRow::text_input(input));
        self
    }
}

// ---------------------------------------------------------------------------
// Component interaction tracking
// ---------------------------------------------------------------------------

/// A component interaction (button click, menu select, modal submit).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentInteraction {
    pub id: String,
    pub custom_id: String,
    pub kind: ComponentInteractionKind,
    pub channel_id: String,
    pub guild_id: Option<String>,
    pub user_id: String,
    pub message_id: Option<String>,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentInteractionKind {
    Button,
    SelectMenu,
    ModalSubmit,
}

/// Manages component interactions and pending modals.
#[derive(Debug)]
pub struct ComponentManager {
    pending_interactions: Arc<Mutex<Vec<ComponentInteraction>>>,
    sent_components: Arc<Mutex<HashMap<String, Vec<ActionRow>>>>,
    pending_modals: Arc<Mutex<HashMap<String, Modal>>>,
}

impl ComponentManager {
    pub fn new() -> Self {
        Self {
            pending_interactions: Arc::new(Mutex::new(Vec::new())),
            sent_components: Arc::new(Mutex::new(HashMap::new())),
            pending_modals: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Track components attached to a message.
    pub async fn attach_components(&self, message_id: &str, rows: Vec<ActionRow>) {
        self.sent_components
            .lock()
            .await
            .insert(message_id.to_string(), rows);
    }

    /// Retrieve components attached to a message.
    pub async fn components_for(&self, message_id: &str) -> Option<Vec<ActionRow>> {
        self.sent_components.lock().await.get(message_id).cloned()
    }

    /// Remove components from a message (e.g. after timeout).
    pub async fn remove_components(&self, message_id: &str) {
        self.sent_components.lock().await.remove(message_id);
    }

    /// Present a modal to a user (stores until submitted).
    pub async fn show_modal(&self, user_id: &str, modal: Modal) {
        self.pending_modals
            .lock()
            .await
            .insert(user_id.to_string(), modal);
    }

    /// Get the pending modal for a user.
    pub async fn pending_modal(&self, user_id: &str) -> Option<Modal> {
        self.pending_modals.lock().await.get(user_id).cloned()
    }

    /// Record an inbound component interaction.
    pub async fn push_interaction(&self, interaction: ComponentInteraction) {
        self.pending_interactions.lock().await.push(interaction);
    }

    /// Pop the next component interaction.
    pub async fn pop_interaction(&self) -> Option<ComponentInteraction> {
        let mut interactions = self.pending_interactions.lock().await;
        if interactions.is_empty() {
            None
        } else {
            Some(interactions.remove(0))
        }
    }

    /// All pending interactions (non-consuming).
    pub async fn pending_interactions(&self) -> Vec<ComponentInteraction> {
        self.pending_interactions.lock().await.clone()
    }

    /// Submit a modal (removes the pending modal and creates a ModalSubmit interaction).
    pub async fn submit_modal(
        &self,
        id: &str,
        user_id: &str,
        channel_id: &str,
        guild_id: Option<&str>,
        values: Vec<String>,
    ) -> Option<ComponentInteraction> {
        let modal = self.pending_modals.lock().await.remove(user_id)?;
        let interaction = ComponentInteraction {
            id: id.to_string(),
            custom_id: modal.custom_id,
            kind: ComponentInteractionKind::ModalSubmit,
            channel_id: channel_id.to_string(),
            guild_id: guild_id.map(ToString::to_string),
            user_id: user_id.to_string(),
            message_id: None,
            values,
        };
        self.pending_interactions
            .lock()
            .await
            .push(interaction.clone());
        Some(interaction)
    }
}

impl Default for ComponentManager {
    fn default() -> Self {
        Self::new()
    }
}
