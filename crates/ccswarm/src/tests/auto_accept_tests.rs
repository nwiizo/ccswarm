use crate::auto_accept::*;
use crate::agent::task::{Task, TaskType, Priority};
use std::path::PathBuf;

#[test]
fn test_auto_accept_config_default() {
    let config = AutoAcceptConfig::default();
    
    // Should be disabled by default for safety
    assert!(!config.enabled);
    assert!(!config.emergency_stop);
    assert!(config.require_tests_pass);
    assert!(config.require_clean_git);
    assert_eq!(config.max_file_changes, 5);
    assert_eq!(config.max_execution_time, 300);
    
    // Should have safe default trusted operations
    assert!(config.trusted_operations.contains(&OperationType::ReadFile));
    assert!(config.trusted_operations.contains(&OperationType::FormatCode));
    assert!(config.trusted_operations.contains(&OperationType::RunTests));
    assert!(config.trusted_operations.contains(&OperationType::LintCode));
    
    // Should have restricted file patterns
    assert!(config.restricted_files.contains(&"Cargo.toml".to_string()));
    assert!(config.restricted_files.contains(&"*.sql".to_string()));
}

#[test]
fn test_operation_type_from_commands() {
    let engine = AutoAcceptEngine::new(AutoAcceptConfig::default());
    
    // Test read operations
    let read_commands = vec!["cat src/main.rs".to_string(), "ls -la".to_string()];
    let operation = engine.analyze_operation(&read_commands, None).unwrap();
    assert_eq!(operation.operation_type, OperationType::ReadFile);
    assert_eq!(operation.risk_level, 1);
    assert!(operation.reversible);
    
    // Test write operations
    let write_commands = vec!["echo 'test' > file.txt".to_string()];
    let operation = engine.analyze_operation(&write_commands, None).unwrap();
    assert_eq!(operation.operation_type, OperationType::WriteFile);
    assert!(operation.risk_level >= 4);
    
    // Test dangerous operations
    let dangerous_commands = vec!["rm -rf /".to_string()];
    let operation = engine.analyze_operation(&dangerous_commands, None).unwrap();
    assert_eq!(operation.operation_type, OperationType::DeleteFile);
    assert!(operation.risk_level >= 8);
    
    // Test test operations
    let test_commands = vec!["cargo test".to_string()];
    let operation = engine.analyze_operation(&test_commands, None).unwrap();
    assert_eq!(operation.operation_type, OperationType::RunTests);
    assert_eq!(operation.risk_level, 2);
    assert!(operation.reversible);
}

#[test]
fn test_should_auto_accept_disabled() {
    let config = AutoAcceptConfig {
        enabled: false,
        ..AutoAcceptConfig::default()
    };
    let engine = AutoAcceptEngine::new(config);
    
    let operation = Operation {
        operation_type: OperationType::ReadFile,
        description: "Read a file".to_string(),
        affected_files: vec![],
        commands: vec!["cat test.txt".to_string()],
        risk_level: 1,
        reversible: true,
        task: None,
    };
    
    let decision = engine.should_auto_accept(&operation).unwrap();
    match decision {
        AutoAcceptDecision::Reject(reason) => {
            assert!(reason.contains("Auto-accept is disabled"));
        }
        _ => panic!("Should reject when disabled"),
    }
}

#[test]
fn test_should_auto_accept_emergency_stop() {
    let config = AutoAcceptConfig {
        enabled: true,
        emergency_stop: true,
        ..AutoAcceptConfig::default()
    };
    let engine = AutoAcceptEngine::new(config);
    
    let operation = Operation {
        operation_type: OperationType::ReadFile,
        description: "Read a file".to_string(),
        affected_files: vec![],
        commands: vec!["cat test.txt".to_string()],
        risk_level: 1,
        reversible: true,
        task: None,
    };
    
    let decision = engine.should_auto_accept(&operation).unwrap();
    match decision {
        AutoAcceptDecision::Reject(reason) => {
            assert!(reason.contains("Emergency stop"));
        }
        _ => panic!("Should reject when emergency stop is active"),
    }
}

#[test]
fn test_should_auto_accept_untrusted_operation() {
    let config = AutoAcceptConfig {
        enabled: true,
        trusted_operations: vec![OperationType::ReadFile], // Only read files trusted
        ..AutoAcceptConfig::default()
    };
    let engine = AutoAcceptEngine::new(config);
    
    let operation = Operation {
        operation_type: OperationType::WriteFile, // Not in trusted list
        description: "Write a file".to_string(),
        affected_files: vec![],
        commands: vec!["echo 'test' > file.txt".to_string()],
        risk_level: 4,
        reversible: false,
        task: None,
    };
    
    let decision = engine.should_auto_accept(&operation).unwrap();
    match decision {
        AutoAcceptDecision::Reject(reason) => {
            assert!(reason.contains("not in trusted operations"));
        }
        _ => panic!("Should reject untrusted operations"),
    }
}

#[test]
fn test_should_auto_accept_high_risk() {
    let config = AutoAcceptConfig {
        enabled: true,
        trusted_operations: vec![OperationType::DeleteFile], // Allow delete for this test
        ..AutoAcceptConfig::default()
    };
    let engine = AutoAcceptEngine::new(config);
    
    let operation = Operation {
        operation_type: OperationType::DeleteFile,
        description: "Delete files".to_string(),
        affected_files: vec![],
        commands: vec!["rm important_file.txt".to_string()],
        risk_level: 8, // High risk
        reversible: false,
        task: None,
    };
    
    let decision = engine.should_auto_accept(&operation).unwrap();
    match decision {
        AutoAcceptDecision::Reject(reason) => {
            assert!(reason.contains("Risk level too high"));
        }
        _ => panic!("Should reject high-risk operations"),
    }
}

#[test]
fn test_should_auto_accept_too_many_files() {
    let config = AutoAcceptConfig {
        enabled: true,
        trusted_operations: vec![OperationType::EditFile],
        max_file_changes: 2, // Low limit for testing
        ..AutoAcceptConfig::default()
    };
    let engine = AutoAcceptEngine::new(config);
    
    let operation = Operation {
        operation_type: OperationType::EditFile,
        description: "Edit multiple files".to_string(),
        affected_files: vec![
            PathBuf::from("file1.rs"),
            PathBuf::from("file2.rs"),
            PathBuf::from("file3.rs"), // 3 files > 2 limit
        ],
        commands: vec!["sed -i 's/old/new/g' *.rs".to_string()],
        risk_level: 3,
        reversible: false,
        task: None,
    };
    
    let decision = engine.should_auto_accept(&operation).unwrap();
    match decision {
        AutoAcceptDecision::Reject(reason) => {
            assert!(reason.contains("Too many file changes"));
        }
        _ => panic!("Should reject when too many files affected"),
    }
}

#[test]
fn test_should_auto_accept_restricted_files() {
    let config = AutoAcceptConfig {
        enabled: true,
        trusted_operations: vec![OperationType::EditFile],
        restricted_files: vec!["Cargo.toml".to_string(), "*.sql".to_string()],
        ..AutoAcceptConfig::default()
    };
    let engine = AutoAcceptEngine::new(config);
    
    // Test exact match restriction
    let operation1 = Operation {
        operation_type: OperationType::EditFile,
        description: "Edit Cargo.toml".to_string(),
        affected_files: vec![PathBuf::from("Cargo.toml")],
        commands: vec!["sed -i 's/old/new/g' Cargo.toml".to_string()],
        risk_level: 3,
        reversible: false,
        task: None,
    };
    
    let decision1 = engine.should_auto_accept(&operation1).unwrap();
    match decision1 {
        AutoAcceptDecision::Reject(reason) => {
            assert!(reason.contains("restricted pattern"));
        }
        _ => panic!("Should reject restricted file Cargo.toml"),
    }
    
    // Test pattern match restriction
    let operation2 = Operation {
        operation_type: OperationType::EditFile,
        description: "Edit SQL file".to_string(),
        affected_files: vec![PathBuf::from("migration.sql")],
        commands: vec!["sed -i 's/old/new/g' migration.sql".to_string()],
        risk_level: 3,
        reversible: false,
        task: None,
    };
    
    let decision2 = engine.should_auto_accept(&operation2).unwrap();
    match decision2 {
        AutoAcceptDecision::Reject(reason) => {
            assert!(reason.contains("restricted pattern"));
        }
        _ => panic!("Should reject restricted pattern *.sql"),
    }
}

#[test]
fn test_should_auto_accept_success() {
    let config = AutoAcceptConfig {
        enabled: true,
        trusted_operations: vec![OperationType::FormatCode, OperationType::LintCode],
        max_file_changes: 10,
        require_tests_pass: true,
        require_clean_git: false, // Disable for testing
        ..AutoAcceptConfig::default()
    };
    let engine = AutoAcceptEngine::new(config);
    
    let operation = Operation {
        operation_type: OperationType::FormatCode,
        description: "Format code".to_string(),
        affected_files: vec![PathBuf::from("src/main.rs")],
        commands: vec!["cargo fmt".to_string()],
        risk_level: 1,
        reversible: true,
        task: None,
    };
    
    let decision = engine.should_auto_accept(&operation).unwrap();
    match decision {
        AutoAcceptDecision::Accept(conditions) => {
            assert!(conditions.contains(&"Tests must pass".to_string()));
        }
        AutoAcceptDecision::Reject(reason) => {
            panic!("Should accept safe operation: {}", reason);
        }
    }
}

#[test]
fn test_pattern_matching() {
    let engine = AutoAcceptEngine::new(AutoAcceptConfig::default());
    
    // Test exact match
    assert!(engine.matches_pattern("Cargo.toml", "Cargo.toml"));
    assert!(!engine.matches_pattern("Cargo.lock", "Cargo.toml"));
    
    // Test suffix wildcard
    assert!(engine.matches_pattern("test.sql", "*.sql"));
    assert!(engine.matches_pattern("migration.sql", "*.sql"));
    assert!(!engine.matches_pattern("test.rs", "*.sql"));
    
    // Test prefix wildcard
    assert!(engine.matches_pattern("config.toml", "*.toml"));
    assert!(!engine.matches_pattern("config.json", "*.toml"));
    
    // Test directory patterns
    assert!(engine.matches_pattern("src/migrations/001.sql", "**/migrations/*"));
    assert!(engine.matches_pattern("db/migrations/init.sql", "**/migrations/*"));
    assert!(!engine.matches_pattern("src/models/user.rs", "**/migrations/*"));
}

#[test]
fn test_emergency_stop_functionality() {
    let config = AutoAcceptConfig {
        enabled: true,
        ..AutoAcceptConfig::default()
    };
    let mut engine = AutoAcceptEngine::new(config);
    
    // Initially enabled
    assert!(engine.config.enabled);
    assert!(!engine.config.emergency_stop);
    
    // Trigger emergency stop
    engine.emergency_stop();
    assert!(!engine.config.enabled);
    assert!(engine.config.emergency_stop);
    
    // Reset emergency stop
    engine.reset_emergency_stop();
    assert!(!engine.config.emergency_stop);
    // Note: enabled remains false after emergency stop - must be manually re-enabled
}

#[test]
fn test_operation_history() {
    let mut engine = AutoAcceptEngine::new(AutoAcceptConfig::default());
    let session_id = "test-session".to_string();
    
    let operation = Operation {
        operation_type: OperationType::ReadFile,
        description: "Test operation".to_string(),
        affected_files: vec![],
        commands: vec!["ls".to_string()],
        risk_level: 1,
        reversible: true,
        task: None,
    };
    
    // Initially no history
    assert!(engine.get_operation_history(&session_id).is_none());
    
    // Record operation
    engine.record_operation(session_id.clone(), operation.clone());
    
    // Should have history now
    let history = engine.get_operation_history(&session_id).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].operation_type, OperationType::ReadFile);
    
    // Clear history
    engine.clear_history(&session_id);
    assert!(engine.get_operation_history(&session_id).is_none());
}

#[test]
fn test_validation_execution_time() {
    let config = AutoAcceptConfig {
        max_execution_time: 60, // 1 minute limit
        ..AutoAcceptConfig::default()
    };
    let engine = AutoAcceptEngine::new(config);
    
    let operation = Operation {
        operation_type: OperationType::ReadFile,
        description: "Test operation".to_string(),
        affected_files: vec![],
        commands: vec!["ls".to_string()],
        risk_level: 1,
        reversible: true,
        task: None,
    };
    
    // Test within time limit
    let result1 = engine.validate_changes(&operation, 30).unwrap();
    match result1 {
        ValidationResult::Valid => {} // Expected
        ValidationResult::Invalid(_) => panic!("Should be valid within time limit"),
    }
    
    // Test exceeding time limit
    let result2 = engine.validate_changes(&operation, 120).unwrap(); // 2 minutes > 1 minute limit
    match result2 {
        ValidationResult::Valid => panic!("Should be invalid when exceeding time limit"),
        ValidationResult::Invalid(issues) => {
            assert!(issues.iter().any(|issue| issue.contains("Execution time exceeded")));
        }
    }
}

#[test]
fn test_task_integration() {
    let engine = AutoAcceptEngine::new(AutoAcceptConfig::default());
    
    let task = Task {
        id: "test-task".to_string(),
        description: "Run tests".to_string(),
        details: Some("Run all unit tests".to_string()),
        priority: Priority::Medium,
        task_type: TaskType::Testing,
        estimated_duration: Some(300),
    };
    
    let commands = vec!["cargo test".to_string()];
    let operation = engine.analyze_operation(&commands, Some(&task)).unwrap();
    
    assert_eq!(operation.operation_type, OperationType::RunTests);
    assert!(operation.task.is_some());
    
    let op_task = operation.task.unwrap();
    assert_eq!(op_task.id, "test-task");
    assert_eq!(op_task.task_type, TaskType::Testing);
}

#[test]
fn test_config_update() {
    let initial_config = AutoAcceptConfig {
        enabled: false,
        max_file_changes: 5,
        ..AutoAcceptConfig::default()
    };
    let mut engine = AutoAcceptEngine::new(initial_config);
    
    assert!(!engine.config.enabled);
    assert_eq!(engine.config.max_file_changes, 5);
    
    let new_config = AutoAcceptConfig {
        enabled: true,
        max_file_changes: 10,
        ..AutoAcceptConfig::default()
    };
    
    engine.update_config(new_config);
    
    assert!(engine.config.enabled);
    assert_eq!(engine.config.max_file_changes, 10);
}