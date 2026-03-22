//! Integration tests: agent turn loop with mock providers.

use std::sync::Arc;

use magicmerlin_agent::{AgentEngine, AgentEngineConfig, SessionKey, SessionManager};
use magicmerlin_integration_tests::{MockTools, SingleShotProvider, TwoStepProvider};
use magicmerlin_providers::model_registry::{ModelCapabilities, ModelDefinition, ModelRegistry};
use magicmerlin_providers::ProviderRouter;
use magicmerlin_storage::Storage;

#[tokio::test]
async fn test_agent_turn_basic() {
    let temp = tempfile::tempdir().expect("tmp");
    let provider = Arc::new(SingleShotProvider::new("Hello! I'm Magic Merlin."));

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
        .run_turn(&mut session, "What is your name?", &MockTools)
        .await
        .expect("run");
    assert_eq!(reply.text, "Hello! I'm Magic Merlin.");
    assert!(reply.rounds >= 1);
}

#[tokio::test]
async fn test_agent_turn_with_tool_call() {
    let temp = tempfile::tempdir().expect("tmp");
    let provider = Arc::new(TwoStepProvider::new("The command output was: hello"));

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
    router.register_provider(provider.clone()).await;

    let storage = Storage::new(temp.path().join("db.sqlite")).expect("storage");
    let sessions =
        SessionManager::new(storage, temp.path().join("sessions"), temp.path()).expect("sessions");

    let mut session = sessions
        .load_or_create(SessionKey::agent_main("merlin"), "merlin")
        .expect("session");

    let engine = AgentEngine::new(
        Arc::clone(&router),
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
        .run_turn(&mut session, "Run echo hello", &MockTools)
        .await
        .expect("run");
    assert_eq!(reply.text, "The command output was: hello");
    // Should have done 2 rounds: tool call + final answer
    assert!(reply.rounds >= 2);
    // Provider should have been called at least twice
    assert!(*provider.calls.lock().unwrap() >= 2);
}

#[tokio::test]
async fn test_agent_turn_token_estimate() {
    let temp = tempfile::tempdir().expect("tmp");
    let provider = Arc::new(SingleShotProvider::new("short reply"));

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
        .run_turn(&mut session, "hello", &MockTools)
        .await
        .expect("run");
    assert!(reply.token_estimate > 0, "token_estimate should be > 0");
}

#[tokio::test]
async fn test_agent_multiple_turns_same_session() {
    let temp = tempfile::tempdir().expect("tmp");
    let provider = Arc::new(SingleShotProvider::new("acknowledged"));

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
        sessions.clone(),
        AgentEngineConfig {
            model: "openai/gpt-5.2".to_string(),
            fallbacks: Vec::new(),
            workspace_dir: temp.path().to_path_buf(),
            agent_dir: temp.path().to_path_buf(),
            ..AgentEngineConfig::default()
        },
    );

    // Run three turns on the same session
    for i in 0..3 {
        let reply = engine
            .run_turn(&mut session, &format!("message {i}"), &MockTools)
            .await
            .expect("run");
        assert_eq!(reply.text, "acknowledged");
    }

    // Session should have accumulated tokens
    assert!(session.token_count > 0);
    // Transcript should have entries for user + assistant * 3
    let messages = sessions.read_messages(&session).expect("read");
    assert!(
        messages.len() >= 6,
        "expected ≥6 transcript entries, got {}",
        messages.len()
    );
}
