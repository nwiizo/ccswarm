//! Template storage implementations

use super::types::{Template, TemplateError, TemplateQuery};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use tracing::{debug, info, warn};

/// Trait for template storage backends
#[async_trait::async_trait]
pub trait TemplateStorage: Send + Sync {
    /// Save a template
    async fn save_template(&mut self, template: &Template) -> Result<(), TemplateError>;

    /// Load a template by ID
    async fn load_template(&self, id: &str) -> Result<Template, TemplateError>;

    /// Delete a template by ID
    async fn delete_template(&mut self, id: &str) -> Result<(), TemplateError>;

    /// List all templates
    async fn list_templates(&self) -> Result<Vec<Template>, TemplateError>;

    /// Search templates by query
    async fn search_templates(&self, query: &TemplateQuery)
    -> Result<Vec<Template>, TemplateError>;

    /// Check if template exists
    async fn exists(&self, id: &str) -> Result<bool, TemplateError>;

    /// Get template statistics
    async fn get_stats(&self) -> Result<TemplateStats, TemplateError>;

    /// Update template usage statistics
    async fn update_usage(&mut self, id: &str, success: bool) -> Result<(), TemplateError>;
}

/// Template storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateStats {
    /// Total number of templates
    pub total_templates: usize,
    /// Templates by category
    pub by_category: HashMap<String, usize>,
    /// Most popular templates
    pub most_popular: Vec<(String, u64)>,
    /// Average success rate
    pub average_success_rate: f64,
    /// Total usage count across all templates
    pub total_usage: u64,
}

/// File system based template storage
pub struct FileSystemTemplateStorage {
    /// Base directory for template storage
    base_dir: PathBuf,
    /// In-memory cache for performance
    cache: HashMap<String, Template>,
    /// Whether cache is valid
    cache_valid: bool,
}

impl FileSystemTemplateStorage {
    /// Create a new file system template storage
    pub async fn new(base_dir: impl AsRef<Path>) -> Result<Self, TemplateError> {
        let base_dir = base_dir.as_ref().to_path_buf();

        // Create directories if they don't exist
        async_fs::create_dir_all(&base_dir)
            .await
            .context("Failed to create template storage directory")?;

        let templates_dir = base_dir.join("templates");
        async_fs::create_dir_all(&templates_dir)
            .await
            .context("Failed to create templates directory")?;

        let mut storage = Self {
            base_dir,
            cache: HashMap::new(),
            cache_valid: false,
        };

        // Load existing templates into cache
        storage.refresh_cache().await?;

        Ok(storage)
    }

    /// Get the templates directory
    fn templates_dir(&self) -> PathBuf {
        self.base_dir.join("templates")
    }

    /// Get the path for a template file
    fn template_path(&self, id: &str) -> PathBuf {
        self.templates_dir().join(format!("{}.json", id))
    }

    /// Refresh the in-memory cache
    async fn refresh_cache(&mut self) -> Result<(), TemplateError> {
        debug!("Refreshing template cache");

        let templates_dir = self.templates_dir();
        if !templates_dir.exists() {
            return Ok(());
        }

        let mut entries = async_fs::read_dir(&templates_dir)
            .await
            .context("Failed to read templates directory")?;

        self.cache.clear();

        while let Some(entry) = entries
            .next_entry()
            .await
            .context("Failed to read directory entry")?
        {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                match self.load_template_from_file(&path).await {
                    Ok(template) => {
                        self.cache.insert(template.id.clone(), template);
                    }
                    Err(e) => {
                        warn!("Failed to load template from {:?}: {}", path, e);
                    }
                }
            }
        }

        self.cache_valid = true;
        info!("Loaded {} templates into cache", self.cache.len());
        Ok(())
    }

    /// Load a template from a file
    async fn load_template_from_file(&self, path: &Path) -> Result<Template, TemplateError> {
        let content = async_fs::read_to_string(path)
            .await
            .context("Failed to read template file")?;

        let template: Template =
            serde_json::from_str(&content).context("Failed to parse template JSON")?;

        Ok(template)
    }

    /// Save a template to file
    async fn save_template_to_file(&self, template: &Template) -> Result<(), TemplateError> {
        let path = self.template_path(&template.id);
        let content =
            serde_json::to_string_pretty(template).context("Failed to serialize template")?;

        async_fs::write(&path, content)
            .await
            .context("Failed to write template file")?;

        Ok(())
    }

    /// Invalidate cache
    #[allow(dead_code)]
    fn invalidate_cache(&mut self) {
        self.cache_valid = false;
    }

    /// Ensure cache is valid
    #[allow(dead_code)]
    async fn ensure_cache_valid(&mut self) -> Result<(), TemplateError> {
        if !self.cache_valid {
            self.refresh_cache().await?;
        }
        Ok(())
    }

    /// Apply query filters to templates
    fn apply_query_filters(
        &self,
        templates: Vec<Template>,
        query: &TemplateQuery,
    ) -> Vec<Template> {
        let mut filtered: Vec<Template> = templates
            .into_iter()
            .filter(|template| {
                // Search term filter
                if let Some(term) = &query.search_term {
                    let term_lower = term.to_lowercase();
                    if !template.name.to_lowercase().contains(&term_lower)
                        && !template.description.to_lowercase().contains(&term_lower)
                        && !template
                            .tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&term_lower))
                    {
                        return false;
                    }
                }

                // Category filter
                if let Some(ref category) = query.category {
                    if template.category != *category {
                        return false;
                    }
                }

                // Tags filter
                if !query.tags.is_empty()
                    && !query.tags.iter().any(|tag| template.tags.contains(tag))
                {
                    return false;
                }

                // Author filter
                if let Some(ref author) = query.author {
                    if template.author.as_ref() != Some(author) {
                        return false;
                    }
                }

                // Success rate filter
                if let Some(min_rate) = query.min_success_rate {
                    if template.success_rate.unwrap_or(0.0) < min_rate {
                        return false;
                    }
                }

                true
            })
            .collect();

        // Apply sorting
        if query.sort_by_popularity {
            filtered.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        } else if query.sort_by_success_rate {
            filtered.sort_by(|a, b| {
                let a_rate = a.success_rate.unwrap_or(0.0);
                let b_rate = b.success_rate.unwrap_or(0.0);
                b_rate
                    .partial_cmp(&a_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        } else if query.sort_by_date {
            filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        }

        // Apply limit
        if let Some(limit) = query.limit {
            filtered.truncate(limit);
        }

        filtered
    }
}

#[async_trait::async_trait]
impl TemplateStorage for FileSystemTemplateStorage {
    async fn save_template(&mut self, template: &Template) -> Result<(), TemplateError> {
        // Validate template
        if !template.is_valid() {
            return Err(TemplateError::ValidationFailed {
                reason: "Template has invalid or missing required fields".to_string(),
            });
        }

        // Check if template already exists
        if self.exists(&template.id).await? {
            return Err(TemplateError::AlreadyExists {
                id: template.id.clone(),
            });
        }

        // Save to file
        self.save_template_to_file(template).await?;

        // Update cache
        self.cache.insert(template.id.clone(), template.clone());

        info!("Saved template: {}", template.id);
        Ok(())
    }

    async fn load_template(&self, id: &str) -> Result<Template, TemplateError> {
        // Try cache first
        if self.cache_valid {
            if let Some(template) = self.cache.get(id) {
                return Ok(template.clone());
            }
        }

        // Load from file
        let path = self.template_path(id);
        if !path.exists() {
            return Err(TemplateError::NotFound { id: id.to_string() });
        }

        self.load_template_from_file(&path).await
    }

    async fn delete_template(&mut self, id: &str) -> Result<(), TemplateError> {
        let path = self.template_path(id);
        if !path.exists() {
            return Err(TemplateError::NotFound { id: id.to_string() });
        }

        // Delete file
        async_fs::remove_file(&path)
            .await
            .context("Failed to delete template file")?;

        // Remove from cache
        self.cache.remove(id);

        info!("Deleted template: {}", id);
        Ok(())
    }

    async fn list_templates(&self) -> Result<Vec<Template>, TemplateError> {
        // For now, just return cached values without mutation
        Ok(self.cache.values().cloned().collect())
    }

    async fn search_templates(
        &self,
        query: &TemplateQuery,
    ) -> Result<Vec<Template>, TemplateError> {
        let all_templates = self.list_templates().await?;
        Ok(self.apply_query_filters(all_templates, query))
    }

    async fn exists(&self, id: &str) -> Result<bool, TemplateError> {
        if self.cache_valid && self.cache.contains_key(id) {
            return Ok(true);
        }

        let path = self.template_path(id);
        Ok(path.exists())
    }

    async fn get_stats(&self) -> Result<TemplateStats, TemplateError> {
        let templates = self.list_templates().await?;

        let mut by_category = HashMap::new();
        let mut total_usage = 0;
        let mut total_success_rate = 0.0;
        let mut templates_with_rate = 0;

        for template in &templates {
            // Count by category
            let category_name = template.category.to_string();
            *by_category.entry(category_name).or_insert(0) += 1;

            // Total usage
            total_usage += template.usage_count;

            // Average success rate
            if let Some(rate) = template.success_rate {
                total_success_rate += rate;
                templates_with_rate += 1;
            }
        }

        // Most popular templates
        let mut popularity: Vec<_> = templates
            .iter()
            .map(|t| (t.name.clone(), t.usage_count))
            .collect();
        popularity.sort_by(|a, b| b.1.cmp(&a.1));
        popularity.truncate(10); // Top 10

        let average_success_rate = if templates_with_rate > 0 {
            total_success_rate / templates_with_rate as f64
        } else {
            0.0
        };

        Ok(TemplateStats {
            total_templates: templates.len(),
            by_category,
            most_popular: popularity,
            average_success_rate,
            total_usage,
        })
    }

    async fn update_usage(&mut self, id: &str, success: bool) -> Result<(), TemplateError> {
        // Load template
        let mut template = self.load_template(id).await?;

        // Update statistics
        template.increment_usage();
        template.update_success_rate(success);

        // Save updated template
        self.save_template_to_file(&template).await?;

        // Update cache
        self.cache.insert(id.to_string(), template);

        Ok(())
    }
}

/// In-memory template storage for testing
pub struct InMemoryTemplateStorage {
    templates: HashMap<String, Template>,
}

impl InMemoryTemplateStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }
}

impl Default for InMemoryTemplateStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl TemplateStorage for InMemoryTemplateStorage {
    async fn save_template(&mut self, template: &Template) -> Result<(), TemplateError> {
        if !template.is_valid() {
            return Err(TemplateError::ValidationFailed {
                reason: "Template has invalid or missing required fields".to_string(),
            });
        }

        if self.templates.contains_key(&template.id) {
            return Err(TemplateError::AlreadyExists {
                id: template.id.clone(),
            });
        }

        self.templates.insert(template.id.clone(), template.clone());
        Ok(())
    }

    async fn load_template(&self, id: &str) -> Result<Template, TemplateError> {
        self.templates
            .get(id)
            .cloned()
            .ok_or_else(|| TemplateError::NotFound { id: id.to_string() })
    }

    async fn delete_template(&mut self, id: &str) -> Result<(), TemplateError> {
        if self.templates.remove(id).is_none() {
            return Err(TemplateError::NotFound { id: id.to_string() });
        }
        Ok(())
    }

    async fn list_templates(&self) -> Result<Vec<Template>, TemplateError> {
        Ok(self.templates.values().cloned().collect())
    }

    async fn search_templates(
        &self,
        query: &TemplateQuery,
    ) -> Result<Vec<Template>, TemplateError> {
        let all_templates = self.list_templates().await?;
        // Reuse the filtering logic from FileSystemTemplateStorage
        let fs_storage = FileSystemTemplateStorage {
            base_dir: PathBuf::new(),
            cache: HashMap::new(),
            cache_valid: true,
        };
        Ok(fs_storage.apply_query_filters(all_templates, query))
    }

    async fn exists(&self, id: &str) -> Result<bool, TemplateError> {
        Ok(self.templates.contains_key(id))
    }

    async fn get_stats(&self) -> Result<TemplateStats, TemplateError> {
        let templates: Vec<_> = self.templates.values().cloned().collect();

        let mut by_category = HashMap::new();
        let mut total_usage = 0;
        let mut total_success_rate = 0.0;
        let mut templates_with_rate = 0;

        for template in &templates {
            let category_name = template.category.to_string();
            *by_category.entry(category_name).or_insert(0) += 1;
            total_usage += template.usage_count;

            if let Some(rate) = template.success_rate {
                total_success_rate += rate;
                templates_with_rate += 1;
            }
        }

        let mut popularity: Vec<_> = templates
            .iter()
            .map(|t| (t.name.clone(), t.usage_count))
            .collect();
        popularity.sort_by(|a, b| b.1.cmp(&a.1));
        popularity.truncate(10);

        let average_success_rate = if templates_with_rate > 0 {
            total_success_rate / templates_with_rate as f64
        } else {
            0.0
        };

        Ok(TemplateStats {
            total_templates: templates.len(),
            by_category,
            most_popular: popularity,
            average_success_rate,
            total_usage,
        })
    }

    async fn update_usage(&mut self, id: &str, success: bool) -> Result<(), TemplateError> {
        let template = self
            .templates
            .get_mut(id)
            .ok_or_else(|| TemplateError::NotFound { id: id.to_string() })?;

        template.increment_usage();
        template.update_success_rate(success);

        Ok(())
    }
}
