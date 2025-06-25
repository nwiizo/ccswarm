//! Tests for the template system

#[cfg(test)]
mod tests {
    use crate::agent::{Priority, TaskType};
    use crate::template::{
        storage::{InMemoryTemplateStorage, TemplateStorage},
        PredefinedTemplates, Template, TemplateCategory, TemplateContext, TemplateManager,
        TemplateVariable,
    };

    #[test]
    fn test_template_creation() {
        let template = Template::new(
            "test-template",
            "Test Template",
            "A test template",
            TemplateCategory::General,
        )
        .with_task_description("Create {{component_name}} component")
        .with_priority(Priority::Medium)
        .with_task_type(TaskType::Development);

        assert_eq!(template.id, "test-template");
        assert_eq!(template.name, "Test Template");
        assert_eq!(
            template.task_description,
            "Create {{component_name}} component"
        );
        assert!(template.is_valid());
    }

    #[test]
    fn test_template_validation() {
        let invalid_template = Template::new("", "", "", TemplateCategory::General);

        assert!(!invalid_template.is_valid());

        let valid_template = Template::new(
            "valid-template",
            "Valid Template",
            "A valid template",
            TemplateCategory::General,
        )
        .with_task_description("Do something");

        assert!(valid_template.is_valid());
    }

    #[test]
    fn test_template_variable() {
        let var = TemplateVariable::text("component_name", "Name of the component")
            .with_example("MyComponent");

        assert_eq!(var.name, "component_name");
        assert_eq!(var.description, "Name of the component");
        assert!(var.required);
        assert_eq!(var.example, Some("MyComponent".to_string()));
    }

    #[tokio::test]
    async fn test_in_memory_storage() {
        let mut storage = InMemoryTemplateStorage::new();

        let template = Template::new(
            "test",
            "Test Template",
            "A test template",
            TemplateCategory::General,
        )
        .with_task_description("Test task");

        // Test save
        storage
            .save_template(&template)
            .await
            .expect("Failed to save template");

        // Test load
        let loaded = storage
            .load_template("test")
            .await
            .expect("Failed to load template");
        assert_eq!(loaded.id, "test");

        // Test list
        let templates = storage
            .list_templates()
            .await
            .expect("Failed to list templates");
        assert_eq!(templates.len(), 1);

        // Test exists
        assert!(storage
            .exists("test")
            .await
            .expect("Failed to check existence"));
        assert!(!storage
            .exists("nonexistent")
            .await
            .expect("Failed to check existence"));
    }

    #[test]
    fn test_predefined_templates() {
        let templates = PredefinedTemplates::get_all();
        assert!(!templates.is_empty());

        // Check that all templates are valid
        for template in &templates {
            assert!(template.is_valid(), "Template {} is invalid", template.id);
        }

        // Check specific templates exist
        let react_component = templates.iter().find(|t| t.id == "react-component");
        assert!(react_component.is_some());

        let api_endpoint = templates.iter().find(|t| t.id == "api-endpoint");
        assert!(api_endpoint.is_some());
    }

    #[tokio::test]
    async fn test_template_manager_basic() {
        let storage = InMemoryTemplateStorage::new();
        let mut manager = TemplateManager::new(storage);

        let template = Template::new(
            "test-manager",
            "Test Manager Template",
            "A template for testing the manager",
            TemplateCategory::General,
        )
        .with_task_description("Create {{name}} with {{description}}")
        .with_variables(vec![
            TemplateVariable::text("name", "The name"),
            TemplateVariable::text("description", "The description"),
        ]);

        // Save template
        manager
            .save_template(template)
            .await
            .expect("Failed to save template");

        // Load template
        let loaded = manager
            .load_template("test-manager")
            .await
            .expect("Failed to load template");
        assert_eq!(loaded.id, "test-manager");

        // Test context and application
        let context = TemplateContext::new()
            .with_variable("name", "TestComponent")
            .with_variable("description", "A test component");

        let applied = manager
            .apply_template("test-manager", context)
            .await
            .expect("Failed to apply template");

        assert_eq!(
            applied.description,
            "Create TestComponent with A test component"
        );
    }
}
