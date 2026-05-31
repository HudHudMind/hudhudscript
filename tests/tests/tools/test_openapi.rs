use hudhudscript_tools::openapi::{
    build_parameters_schema, derive_tool_name, discover_tools_from_openapi, extract_param_type,
    import_openapi_tools, OpenApiDocument, OpenApiOperation,
};
use hudhudscript_tools::registry::ToolRegistry;

const PETSTORE_MINI: &str = r#"
{
    "openapi": "3.1.0",
    "info": {
        "title": "Petstore",
        "version": "1.0.0"
    },
    "paths": {
        "/pets": {
            "get": {
                "operationId": "listPets",
                "summary": "List all pets",
                "parameters": [
                    {
                        "name": "limit",
                        "in": "query",
                        "required": false,
                        "schema": { "type": "integer" }
                    }
                ]
            },
            "post": {
                "operationId": "createPet",
                "summary": "Create a pet",
                "requestBody": {
                    "required": true,
                    "content": {
                        "application/json": {
                            "schema": {
                                "type": "object",
                                "properties": {
                                    "name": { "type": "string", "description": "Pet name" },
                                    "species": { "type": "string" }
                                },
                                "required": ["name"]
                            }
                        }
                    }
                }
            }
        },
        "/pets/{petId}": {
            "get": {
                "operationId": "getPetById",
                "summary": "Get a pet by ID",
                "parameters": [
                    {
                        "name": "petId",
                        "in": "path",
                        "required": true,
                        "schema": { "type": "string" }
                    }
                ]
            }
        }
    }
}
"#;

#[test]
fn test_discover_tools_count() {
    let tools = discover_tools_from_openapi(PETSTORE_MINI).unwrap();
    assert_eq!(tools.len(), 3, "Expected 3 tools from petstore mini spec");
}

#[test]
fn test_tool_names_derived_from_operation_id() {
    let tools = discover_tools_from_openapi(PETSTORE_MINI).unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"listpets"), "Expected 'listpets'");
    assert!(names.contains(&"createpet"), "Expected 'createpet'");
    assert!(names.contains(&"getpetbyid"), "Expected 'getpetbyid'");
}

#[test]
fn test_parameter_extraction() {
    let tools = discover_tools_from_openapi(PETSTORE_MINI).unwrap();
    let list_pets = tools.iter().find(|t| t.name == "listpets").unwrap();
    let props = list_pets.parameters.properties.as_ref().unwrap();
    assert!(props.contains_key("limit"));
    assert_eq!(props["limit"].property_type, "integer");
}

#[test]
fn test_request_body_flattening() {
    let tools = discover_tools_from_openapi(PETSTORE_MINI).unwrap();
    let create_pet = tools.iter().find(|t| t.name == "createpet").unwrap();
    let props = create_pet.parameters.properties.as_ref().unwrap();
    assert!(props.contains_key("name"));
    assert!(props.contains_key("species"));
    // "name" should be in required
    let req = create_pet.parameters.required.as_ref().unwrap();
    assert!(req.contains(&"name".to_string()));
}

#[test]
fn test_path_param_required() {
    let tools = discover_tools_from_openapi(PETSTORE_MINI).unwrap();
    let get_pet = tools.iter().find(|t| t.name == "getpetbyid").unwrap();
    let req = get_pet.parameters.required.as_ref().unwrap();
    assert!(req.contains(&"petId".to_string()));
}

#[test]
fn test_invalid_json_returns_error() {
    let err = discover_tools_from_openapi("not json at all");
    assert!(err.is_err());
}

#[test]
fn test_import_into_registry() {
    let registry = ToolRegistry::new();
    let count = import_openapi_tools(&registry, PETSTORE_MINI, "petstore").unwrap();
    assert_eq!(count, 3);
    assert!(registry.get_tool("listpets").is_some());
    assert!(registry.get_tool("createpet").is_some());
    assert!(registry.get_tool("getpetbyid").is_some());
}

#[test]
fn test_sanitize_name() {
    use hudhudscript_tools::openapi::sanitize_name;
    assert_eq!(sanitize_name("listPets"), "listpets");
    assert_eq!(sanitize_name("get-pet-by-id"), "get_pet_by_id");
    assert_eq!(sanitize_name("getPet/123"), "getpet_123");
}

// ---- derive_tool_name without operationId falls back to method+path ----

#[test]
fn test_derive_tool_name_no_operation_id() {
    let op = OpenApiOperation {
        operation_id: None,
        summary: None,
        description: None,
        parameters: vec![],
        request_body: None,
        tags: vec![],
    };
    let name = derive_tool_name(&op, "GET", "/users/{id}/orders");
    assert_eq!(name, "get_users_id_orders");
}

#[test]
fn test_derive_tool_name_with_operation_id() {
    let op = OpenApiOperation {
        operation_id: Some("getUserOrders".to_string()),
        summary: None,
        description: None,
        parameters: vec![],
        request_body: None,
        tags: vec![],
    };
    let name = derive_tool_name(&op, "GET", "/users/{id}/orders");
    assert_eq!(name, "getuserorders");
}

// ---- sanitize_name edge cases ----

#[test]
fn test_sanitize_name_all_special_chars() {
    use hudhudscript_tools::openapi::sanitize_name;
    assert_eq!(sanitize_name("a.b:c@d!e"), "a_b_c_d_e");
}

#[test]
fn test_sanitize_name_underscores_preserved() {
    use hudhudscript_tools::openapi::sanitize_name;
    assert_eq!(sanitize_name("my_tool_name"), "my_tool_name");
}

#[test]
fn test_sanitize_name_empty() {
    use hudhudscript_tools::openapi::sanitize_name;
    assert_eq!(sanitize_name(""), "");
}

// ---- discover_tools_from_openapi with empty paths ----

#[test]
fn test_discover_tools_empty_paths() {
    let json = r#"{"openapi":"3.1.0","info":{"title":"Empty"},"paths":{}}"#;
    let tools = discover_tools_from_openapi(json).unwrap();
    assert_eq!(tools.len(), 0);
}

// ---- discover_tools with description fallback ----

#[test]
fn test_discover_tools_description_fallback() {
    let json = r#"{
        "openapi":"3.1.0",
        "paths":{
            "/test":{
                "get":{
                    "operationId":"testOp",
                    "description":"A longer description"
                }
            }
        }
    }"#;
    let tools = discover_tools_from_openapi(json).unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(
        tools[0].description.as_deref(),
        Some("A longer description")
    );
}

// ---- extract_param_type with missing schema ----

#[test]
fn test_extract_param_type_none_schema() {
    let result = extract_param_type(None);
    assert_eq!(result, "string"); // default
}

#[test]
fn test_extract_param_type_no_type_field() {
    let schema = serde_json::json!({"format": "int32"});
    let result = extract_param_type(Some(&schema));
    assert_eq!(result, "string"); // default when type missing
}

#[test]
fn test_extract_param_type_integer() {
    let schema = serde_json::json!({"type": "integer"});
    let result = extract_param_type(Some(&schema));
    assert_eq!(result, "integer");
}

// ---- OpenApiDocument with swagger field ----

#[test]
fn test_openapi_document_swagger_field() {
    let json = r#"{"swagger":"2.0","info":{"title":"Old API"},"paths":{}}"#;
    let doc: OpenApiDocument = serde_json::from_str(json).unwrap();
    assert_eq!(doc.swagger.as_deref(), Some("2.0"));
    assert!(doc.openapi.is_none());
}

// ---- import_openapi_tools with metadata tags ----

#[test]
fn test_import_openapi_tools_preserves_tags() {
    let json = r#"{
        "openapi":"3.1.0",
        "paths":{
            "/tagged":{
                "get":{
                    "operationId":"taggedOp",
                    "summary":"Tagged operation",
                    "tags":["pets","admin"]
                }
            }
        }
    }"#;
    let registry = ToolRegistry::new();
    import_openapi_tools(&registry, json, "tag-server").unwrap();
    let meta = registry.get_metadata("taggedop").unwrap();
    assert!(meta.tags.contains(&"pets".to_string()));
    assert!(meta.tags.contains(&"admin".to_string()));
}

// ---- build_parameters_schema with no parameters and no body ----

#[test]
fn test_build_parameters_schema_empty() {
    let op = OpenApiOperation {
        operation_id: None,
        summary: None,
        description: None,
        parameters: vec![],
        request_body: None,
        tags: vec![],
    };
    let schema = build_parameters_schema(&op);
    assert_eq!(schema.schema_type, "object");
    assert!(schema.properties.is_none());
    assert!(schema.required.is_none());
}
