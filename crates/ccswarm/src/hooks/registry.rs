//! Hook Registry
//!
//! Central registry for managing and invoking hooks in order of priority.

use super::*;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Registry for managing hooks
pub struct HookRegistry {
    execution_hooks: Arc<RwLock<Vec<Arc<dyn ExecutionHooks>>>>,
    tool_hooks: Arc<RwLock<Vec<Arc<dyn ToolHooks>>>>,
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HookRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            execution_hooks: Arc::new(RwLock::new(Vec::new())),
            tool_hooks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a registry with default hooks (Logging, Security)
    pub fn with_defaults() -> Self {
        let registry = Self::new();
        // Note: Actual registration happens asynchronously
        registry
    }

    /// Register an execution hook
    pub async fn register_execution_hook(&self, hook: Arc<dyn ExecutionHooks>) {
        let mut hooks = self.execution_hooks.write().await;
        hooks.push(hook);
        // Sort by priority (descending - higher priority first)
        hooks.sort_by_key(|h| std::cmp::Reverse(h.priority()));
    }

    /// Register a tool hook
    pub async fn register_tool_hook(&self, hook: Arc<dyn ToolHooks>) {
        let mut hooks = self.tool_hooks.write().await;
        hooks.push(hook);
        hooks.sort_by_key(|h| std::cmp::Reverse(h.priority()));
    }

    /// Unregister an execution hook by name
    pub async fn unregister_execution_hook(&self, name: &str) {
        let mut hooks = self.execution_hooks.write().await;
        hooks.retain(|h| h.name() != name);
    }

    /// Unregister a tool hook by name
    pub async fn unregister_tool_hook(&self, name: &str) {
        let mut hooks = self.tool_hooks.write().await;
        hooks.retain(|h| h.name() != name);
    }

    /// List registered execution hooks
    pub async fn list_execution_hooks(&self) -> Vec<String> {
        let hooks = self.execution_hooks.read().await;
        hooks.iter().map(|h| h.name().to_string()).collect()
    }

    /// List registered tool hooks
    pub async fn list_tool_hooks(&self) -> Vec<String> {
        let hooks = self.tool_hooks.read().await;
        hooks.iter().map(|h| h.name().to_string()).collect()
    }

    /// Run pre-execution hooks
    ///
    /// Returns the first non-Continue result or Continue if all hooks pass
    pub async fn run_pre_execution(
        &self,
        input: PreExecutionInput,
        ctx: HookContext,
    ) -> HookResult {
        let hooks = self.execution_hooks.read().await;
        for hook in hooks.iter() {
            let result = hook.pre_execution(input.clone(), ctx.clone()).await;
            if !result.should_continue() {
                tracing::info!(
                    hook_name = hook.name(),
                    "Pre-execution hook blocked operation"
                );
                return result;
            }
        }
        HookResult::Continue
    }

    /// Run post-execution hooks
    pub async fn run_post_execution(
        &self,
        input: PostExecutionInput,
        ctx: HookContext,
    ) -> HookResult {
        let hooks = self.execution_hooks.read().await;
        for hook in hooks.iter() {
            let result = hook.post_execution(input.clone(), ctx.clone()).await;
            if !result.should_continue() {
                tracing::info!(
                    hook_name = hook.name(),
                    "Post-execution hook blocked operation"
                );
                return result;
            }
        }
        HookResult::Continue
    }

    /// Run error hooks
    pub async fn run_on_error(&self, input: OnErrorInput, ctx: HookContext) -> HookResult {
        let hooks = self.execution_hooks.read().await;
        for hook in hooks.iter() {
            let result = hook.on_error(input.clone(), ctx.clone()).await;
            if !result.should_continue() {
                tracing::info!(hook_name = hook.name(), "Error hook modified handling");
                return result;
            }
        }
        HookResult::Continue
    }

    /// Run pre-tool-use hooks
    pub async fn run_pre_tool_use(&self, input: PreToolUseInput, ctx: HookContext) -> HookResult {
        let hooks = self.tool_hooks.read().await;
        for hook in hooks.iter() {
            let result = hook.pre_tool_use(input.clone(), ctx.clone()).await;
            if !result.should_continue() {
                tracing::info!(
                    hook_name = hook.name(),
                    tool_name = %input.tool_name,
                    "Pre-tool-use hook blocked operation"
                );
                return result;
            }
        }
        HookResult::Continue
    }

    /// Run post-tool-use hooks
    pub async fn run_post_tool_use(&self, input: PostToolUseInput, ctx: HookContext) -> HookResult {
        let hooks = self.tool_hooks.read().await;
        for hook in hooks.iter() {
            let result = hook.post_tool_use(input.clone(), ctx.clone()).await;
            if !result.should_continue() {
                tracing::info!(
                    hook_name = hook.name(),
                    tool_name = %input.tool_name,
                    "Post-tool-use hook blocked operation"
                );
                return result;
            }
        }
        HookResult::Continue
    }
}

impl Clone for HookRegistry {
    fn clone(&self) -> Self {
        Self {
            execution_hooks: Arc::clone(&self.execution_hooks),
            tool_hooks: Arc::clone(&self.tool_hooks),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHook {
        name: String,
        priority: i32,
    }

    #[async_trait]
    impl ExecutionHooks for TestHook {
        async fn pre_execution(&self, _input: PreExecutionInput, _ctx: HookContext) -> HookResult {
            HookResult::Continue
        }

        async fn post_execution(
            &self,
            _input: PostExecutionInput,
            _ctx: HookContext,
        ) -> HookResult {
            HookResult::Continue
        }

        async fn on_error(&self, _input: OnErrorInput, _ctx: HookContext) -> HookResult {
            HookResult::Continue
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn priority(&self) -> i32 {
            self.priority
        }
    }

    #[tokio::test]
    async fn test_hook_registration() {
        let registry = HookRegistry::new();

        let hook = Arc::new(TestHook {
            name: "test".to_string(),
            priority: 0,
        });

        registry.register_execution_hook(hook).await;

        let hooks = registry.list_execution_hooks().await;
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0], "test");
    }

    #[tokio::test]
    async fn test_hook_priority_ordering() {
        let registry = HookRegistry::new();

        let low = Arc::new(TestHook {
            name: "low".to_string(),
            priority: 0,
        });
        let high = Arc::new(TestHook {
            name: "high".to_string(),
            priority: 100,
        });

        registry.register_execution_hook(low).await;
        registry.register_execution_hook(high).await;

        let hooks = registry.list_execution_hooks().await;
        assert_eq!(hooks[0], "high"); // Higher priority first
        assert_eq!(hooks[1], "low");
    }
}
