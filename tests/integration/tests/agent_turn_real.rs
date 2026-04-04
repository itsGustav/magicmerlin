//! Integration test: agent turn end-to-end — verifies the AgentEngine produces
//! real (non-echo-stub) responses via mock providers.

use std::sync::Arc;

use magicmerlin_agent::{AgentEngine, AgentEngineConfig, SessionKey, SessionManager};
use magicmerlin_integration_tests::{MockTools, SingleShotProvider};
use magicmerlin_providers::model_registry::{ModelCapabilities, ModelDefinition, ModelRegistry};
use magicmerlin_providers::ProviderRouter;
use magicmerlin_storage::Storage;

/// Verify that the agent turn produces a real response, NOT the old echo stub
/// pattern "Echo: <user_message>".
#[tokio::test]
async fn test_agent_turn_real_response() {
    let temp = tempfile::tempdir().expect("tmp");

    // The provider returns a realistic reply — nothing like "Echo: …".
    let provider = Arc::new(SingleShotProvider::new(
        "Sure! The word is: TESTMARKER. Anything else?",
    ));

    let mut models = ModelRegistry::default();
    models.upsert_model(ModelDefinition {
        provider: "openai".to_string(),
        model_id: "gpt-5.2".to_string(),
        context_window: 128_000,
        max_tokens: 8_192,
        input_cost_per_mtok: 0.0,
        output_cost_per_mtok: 0.0,
        capabilities: ModelCapabilities::default(),
    });
    let router = Arc::new(ProviderRouter::new(models));
    router.register_provider(provider).await;

    let storage = Storage::new(temp.path().join("db.sqlite")).expect("storage");
    let sessions =
        SessionManager::new(storage, temp.path().join("sessions"), temp.path()).expect("sessions");

    let mut session = sessions
        .load_or_create(SessionKey::agent_main("merlin"), "merlin")
        .expect("session");

    let engine = AgentEngine::new(
        router,
        sessions,
        AgentEngineConfig {
            model: "openai/gpt-5.2".to_string(),
            fallbacks: Vec::new(),
            workspace_dir: temp.path().to_path_buf(),
            agent_dir: temp.path().to_path_buf(),
            ..AgentEngineConfig::default()
        },
    );

    let reply = engine
        .run_turn(&mut session, "Say the word TESTMARKER", &MockTools)
        .await
        .expect("run_turn");

    // The reply must contain TESTMARKER — verifying the provider was actually called.
    assert!(
        reply.text.contains("TESTMARKER"),
        "reply should contain TESTMARKER, got: {}",
        reply.text
    );

    // The reply must NOT be the old echo stub format.
    assert!(
        !reply.text.starts_with("Echo:"),
        "reply looks like old echo stub: {}",
        reply.text
    );
}
