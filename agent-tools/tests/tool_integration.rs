use serde_json::{json, Value};

use magicmerlin_agent_tools::register_default_tools;
use magicmerlin_agent_tools::{ToolRegistry, ToolResult};

// ── Registry Tests ──

#[test]
fn test_registry_has_default_tools() {
    let mut registry = ToolRegistry::new();
    register_default_tools(&mut registry);
    let names = registry.names();
    assert!(names.len() >= 20, "Expected 20+ tools, got {}", names.len());
}

#[test]
fn test_registry_contains_core_tools() {
    let mut registry = ToolRegistry::new();
    register_default_tools(&mut registry);
    let names = registry.names();
    let expected = vec![
        "exec",
        "read",
        "write",
        "edit",
        "web_search",
        "web_fetch",
        "memory_search",
        "message",
        "cron",
    ];
    for name in expected {
        assert!(names.iter().any(|n| n == name), "Missing tool: {name}");
    }
}

#[test]
fn test_registry_schemas_are_valid_json() {
    let mut registry = ToolRegistry::new();
    register_default_tools(&mut registry);
    let schemas: Vec<Value> = registry.schemas();
    assert!(!schemas.is_empty());
    for schema in &schemas {
        assert!(schema.is_object(), "Schema entry should be a JSON object");
        // Every schema entry should have name, description, parameters
        let name = schema
            .get("name")
            .and_then(|v: &Value| v.as_str())
            .unwrap_or("");
        assert!(!name.is_empty(), "Schema entry missing 'name' field");
        assert!(
            schema.get("description").is_some(),
            "Schema for {name} missing 'description' field"
        );
        assert!(
            schema.get("parameters").is_some(),
            "Schema for {name} missing 'parameters' field"
        );
    }
}

#[test]
fn test_registry_deny_tool() {
    let mut registry = ToolRegistry::new();
    register_default_tools(&mut registry);
    let initial_count = registry.names().len();
    registry.deny_tool("exec");
    let names = registry.names();
    // deny_tool does not remove the tool, it only blocks execute
    assert_eq!(names.len(), initial_count);
}

#[test]
fn test_tool_result_success() {
    let result = ToolResult::success(json!({"hello": "world"}));
    assert!(result.ok);
    assert_eq!(result.value["hello"], "world");
    assert!(!result.truncated);
}

#[test]
fn test_tool_result_failure() {
    let result = ToolResult::failure("something went wrong");
    assert!(!result.ok);
    assert_eq!(
        result.value["error"].as_str().unwrap(),
        "something went wrong"
    );
}

// ── Schema shape tests for individual tools ──

fn find_schema(schemas: &[serde_json::Value], tool_name: &str) -> serde_json::Value {
    schemas
        .iter()
        .find(|s| s["name"].as_str() == Some(tool_name))
        .unwrap_or_else(|| panic!("Schema not found for tool: {tool_name}"))
        .clone()
}

#[test]
fn test_exec_tool_schema() {
    let mut registry = ToolRegistry::new();
    register_default_tools(&mut registry);
    let schemas = registry.schemas();
    let schema = find_schema(&schemas, "exec");
    let params = &schema["parameters"];
    let props = params
        .get("properties")
        .expect("exec should have properties");
    assert!(
        props.get("cmd").is_some(),
        "exec should have 'cmd' property"
    );
}

#[test]
fn test_read_tool_schema() {
    let mut registry = ToolRegistry::new();
    register_default_tools(&mut registry);
    let schemas = registry.schemas();
    let schema = find_schema(&schemas, "read");
    let params = &schema["parameters"];
    let props = params
        .get("properties")
        .expect("read should have properties");
    assert!(
        props.get("path").is_some(),
        "read should have 'path' property"
    );
}

#[test]
fn test_write_tool_schema() {
    let mut registry = ToolRegistry::new();
    register_default_tools(&mut registry);
    let schemas = registry.schemas();
    let schema = find_schema(&schemas, "write");
    let params = &schema["parameters"];
    let props = params
        .get("properties")
        .expect("write should have properties");
    assert!(props.get("path").is_some());
    assert!(props.get("content").is_some());
}

#[test]
fn test_memory_search_tool_schema() {
    let mut registry = ToolRegistry::new();
    register_default_tools(&mut registry);
    let schemas = registry.schemas();
    let schema = find_schema(&schemas, "memory_search");
    let params = &schema["parameters"];
    let props = params
        .get("properties")
        .expect("memory_search should have properties");
    assert!(props.get("query").is_some());
}

#[test]
fn test_web_fetch_tool_schema() {
    let mut registry = ToolRegistry::new();
    register_default_tools(&mut registry);
    let schemas = registry.schemas();
    let schema = find_schema(&schemas, "web_fetch");
    let params = &schema["parameters"];
    let props = params
        .get("properties")
        .expect("web_fetch should have properties");
    assert!(props.get("url").is_some());
}

#[test]
fn test_all_tools_have_descriptions() {
    let mut registry = ToolRegistry::new();
    register_default_tools(&mut registry);
    let schemas = registry.schemas();
    for schema in &schemas {
        let name = schema["name"].as_str().unwrap_or("");
        assert!(!name.is_empty(), "Tool name should not be empty");
        let desc = schema["description"].as_str().unwrap_or("");
        assert!(!desc.is_empty(), "Tool '{name}' should have a description");
    }
}

#[test]
fn test_browser_tool_registered() {
    let mut registry = ToolRegistry::new();
    register_default_tools(&mut registry);
    let names = registry.names();
    assert!(
        names.iter().any(|n| n == "browser"),
        "browser tool should be registered"
    );
}

#[test]
fn test_image_tool_registered() {
    let mut registry = ToolRegistry::new();
    register_default_tools(&mut registry);
    let names = registry.names();
    assert!(
        names.iter().any(|n| n == "image"),
        "image tool should be registered"
    );
}

#[test]
fn test_pdf_tool_registered() {
    let mut registry = ToolRegistry::new();
    register_default_tools(&mut registry);
    let names = registry.names();
    assert!(
        names.iter().any(|n| n == "pdf"),
        "pdf tool should be registered"
    );
}

#[test]
fn test_tts_tool_registered() {
    let mut registry = ToolRegistry::new();
    register_default_tools(&mut registry);
    let names = registry.names();
    assert!(
        names.iter().any(|n| n == "tts"),
        "tts tool should be registered"
    );
}

#[test]
fn test_canvas_tool_registered() {
    let mut registry = ToolRegistry::new();
    register_default_tools(&mut registry);
    let names = registry.names();
    assert!(
        names.iter().any(|n| n == "canvas"),
        "canvas tool should be registered"
    );
}

#[test]
fn test_nodes_tool_registered() {
    let mut registry = ToolRegistry::new();
    register_default_tools(&mut registry);
    let names = registry.names();
    assert!(
        names.iter().any(|n| n == "nodes"),
        "nodes tool should be registered"
    );
}

#[test]
fn test_schemas_sorted_by_name() {
    let mut registry = ToolRegistry::new();
    register_default_tools(&mut registry);
    let schemas = registry.schemas();
    let names: Vec<&str> = schemas.iter().filter_map(|s| s["name"].as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(
        names, sorted,
        "schemas() should return entries sorted by name"
    );
}

#[test]
fn test_empty_registry() {
    let registry = ToolRegistry::new();
    assert!(registry.names().is_empty());
    assert!(registry.schemas().is_empty());
}

#[test]
fn test_max_result_bytes_default() {
    let registry = ToolRegistry::new();
    assert_eq!(registry.max_result_bytes, 64 * 1024);
}
