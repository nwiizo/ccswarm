/// Advanced macro utilities for code generation and pattern reduction

/// Macro for generating message types with automatic builder pattern
#[macro_export]
macro_rules! define_messages {
    (
        $(
            $(#[$meta:meta])*
            $variant:ident {
                $(
                    $(#[$field_meta:meta])*
                    $field:ident: $type:ty
                ),* $(,)?
            }
        )*
    ) => {
        use serde::{Serialize, Deserialize};
        use uuid::Uuid;
        use std::collections::HashMap;

        /// Unified message enum containing all message types
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(tag = "type")]
        pub enum UnifiedMessage {
            $(
                $(#[$meta])*
                $variant($variant),
            )*
        }

        $(
            $(#[$meta])*
            #[derive(Debug, Clone, Serialize, Deserialize, Default)]
            pub struct $variant {
                /// Unique message ID
                #[serde(default = "generate_id")]
                pub id: String,
                
                $(
                    $(#[$field_meta])*
                    pub $field: $type,
                )*
                
                /// Message timestamp
                #[serde(default = "chrono::Utc::now")]
                pub timestamp: chrono::DateTime<chrono::Utc>,
            }

            impl $variant {
                /// Create new message with builder pattern
                pub fn builder() -> paste::paste! { [<$variant Builder>] } {
                    paste::paste! { [<$variant Builder>]::default() }
                }
                
                /// Quick constructor for simple cases
                pub fn new($($field: $type),*) -> Self {
                    Self {
                        id: generate_id(),
                        $($field,)*
                        timestamp: chrono::Utc::now(),
                    }
                }
                
                /// Convert to UnifiedMessage
                pub fn into_unified(self) -> UnifiedMessage {
                    UnifiedMessage::$variant(self)
                }
            }

            paste::paste! {
                #[derive(Default)]
                pub struct [<$variant Builder>] {
                    id: Option<String>,
                    $($field: Option<$type>,)*
                    timestamp: Option<chrono::DateTime<chrono::Utc>>,
                }
                
                impl [<$variant Builder>] {
                    pub fn id(mut self, id: impl Into<String>) -> Self {
                        self.id = Some(id.into());
                        self
                    }
                    
                    $(
                        pub fn $field(mut self, $field: $type) -> Self {
                            self.$field = Some($field);
                            self
                        }
                    )*
                    
                    pub fn timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
                        self.timestamp = Some(timestamp);
                        self
                    }
                    
                    pub fn build(self) -> Result<$variant, &'static str> {
                        Ok($variant {
                            id: self.id.unwrap_or_else(generate_id),
                            $(
                                $field: self.$field.ok_or(concat!("Missing field: ", stringify!($field)))?,
                            )*
                            timestamp: self.timestamp.unwrap_or_else(chrono::Utc::now),
                        })
                    }
                }
            }
        )*

        fn generate_id() -> String {
            Uuid::new_v4().to_string()
        }
    };
}

/// Macro for defining async state machines with automatic transition handling
#[macro_export]
macro_rules! async_state_machine {
    (
        machine: $machine:ident,
        context: $context:ty,
        error: $error:ty,
        states: {
            $(
                $state:ident $(($($state_data:ty),*))? {
                    $(
                        on $event:ident $(($($event_data:ident: $event_type:ty),*))? => $next_state:ident
                        $transition_body:block
                    )*
                }
            ),* $(,)?
        }
    ) => {
        use async_trait::async_trait;
        use std::collections::HashMap;
        use std::sync::Arc;
        use tokio::sync::RwLock;

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum State {
            $(
                $state $(($($state_data),*))?
            ),*
        }

        #[derive(Debug, Clone)]
        pub enum Event {
            $(
                $(
                    $event $(($($event_type),*))?
                ),*
            ),*
        }

        pub struct $machine {
            state: Arc<RwLock<State>>,
            context: Arc<RwLock<$context>>,
            transition_hooks: Arc<RwLock<TransitionHooks>>,
        }

        struct TransitionHooks {
            on_enter: HashMap<State, Box<dyn Fn(&$context) + Send + Sync>>,
            on_exit: HashMap<State, Box<dyn Fn(&$context) + Send + Sync>>,
        }

        impl $machine {
            pub fn new(initial_context: $context) -> Self {
                Self {
                    state: Arc::new(RwLock::new(State::default())),
                    context: Arc::new(RwLock::new(initial_context)),
                    transition_hooks: Arc::new(RwLock::new(TransitionHooks {
                        on_enter: HashMap::new(),
                        on_exit: HashMap::new(),
                    })),
                }
            }

            pub async fn handle_event(&self, event: Event) -> Result<(), $error> {
                let current_state = self.state.read().await.clone();
                
                match (&current_state, &event) {
                    $(
                        $(
                            (State::$state $(($($state_data),*))?, Event::$event $(($($event_data),*))?) => {
                                self.transition_to(State::$next_state, || async {
                                    $transition_body
                                }).await?;
                            }
                        )*
                    ),*
                    _ => return Err(anyhow::anyhow!("Invalid transition from {:?} on {:?}", current_state, event).into()),
                }
                
                Ok(())
            }

            async fn transition_to<F, Fut>(&self, new_state: State, transition_fn: F) -> Result<(), $error>
            where
                F: FnOnce() -> Fut,
                Fut: std::future::Future<Output = Result<(), $error>>,
            {
                let old_state = {
                    let mut state = self.state.write().await;
                    let old = state.clone();
                    *state = new_state.clone();
                    old
                };

                // Execute exit hook
                if let Some(on_exit) = self.transition_hooks.read().await.on_exit.get(&old_state) {
                    on_exit(&*self.context.read().await);
                }

                // Execute transition
                transition_fn().await?;

                // Execute enter hook
                if let Some(on_enter) = self.transition_hooks.read().await.on_enter.get(&new_state) {
                    on_enter(&*self.context.read().await);
                }

                info!("Transitioned from {:?} to {:?}", old_state, new_state);
                Ok(())
            }

            pub async fn current_state(&self) -> State {
                self.state.read().await.clone()
            }

            pub async fn with_context<F, R>(&self, f: F) -> R
            where
                F: FnOnce(&$context) -> R,
            {
                f(&*self.context.read().await)
            }

            pub async fn with_context_mut<F, R>(&self, f: F) -> R
            where
                F: FnOnce(&mut $context) -> R,
            {
                f(&mut *self.context.write().await)
            }
        }

        impl Default for State {
            fn default() -> Self {
                // First state is the default
                $(
                    return State::$state $((<$($state_data>::default()),*))?;
                )*
            }
        }
    };
}

/// Macro for generating pattern DSL
#[macro_export]
macro_rules! pattern_dsl {
    (
        $(
            pattern $name:ident {
                triggers: [$($trigger:expr),* $(,)?],
                confidence: $confidence:expr,
                tasks: [
                    $(
                        {
                            description: $desc:expr,
                            task_type: $task_type:ident,
                            priority: $priority:ident,
                            duration: $duration:expr,
                            agent: $agent:expr $(,)?
                        }
                    ),* $(,)?
                ] $(,)?
            }
        )*
    ) => {{
        use std::collections::HashMap;
        
        let mut patterns = HashMap::new();
        
        $(
            patterns.insert(
                stringify!($name).to_string(),
                $crate::orchestrator::TaskPattern {
                    pattern_id: stringify!($name).to_string(),
                    trigger_conditions: vec![$($trigger.to_string()),*],
                    generated_tasks: vec![
                        $(
                            $crate::orchestrator::TaskTemplate {
                                description_template: $desc.to_string(),
                                task_type: $crate::task::TaskType::$task_type,
                                priority: $crate::task::Priority::$priority,
                                estimated_duration: $duration,
                                required_agent_type: $agent.to_string(),
                                variables: HashMap::new(),
                            }
                        ),*
                    ],
                    confidence: $confidence,
                    usage_count: 0,
                }
            );
        )*
        
        patterns
    }};
}

/// Macro for async operation with automatic retry, timeout, and error handling
#[macro_export]
macro_rules! async_operation {
    (
        name: $name:expr,
        timeout: $timeout:expr,
        retries: $retries:expr,
        $body:block
    ) => {{
        use tokio::time::{timeout, Duration};
        use tracing::{instrument, error, debug};
        
        #[instrument(name = $name, skip_all)]
        async fn operation() -> Result<_, Box<dyn std::error::Error>> {
            $body
        }
        
        let mut attempts = 0;
        let max_retries = $retries;
        let timeout_duration = Duration::from_secs($timeout);
        
        loop {
            attempts += 1;
            debug!("Attempt {} of {}", attempts, max_retries + 1);
            
            match timeout(timeout_duration, operation()).await {
                Ok(Ok(result)) => {
                    metrics::counter!(concat!($name, ".success"), 1);
                    break Ok(result);
                }
                Ok(Err(e)) if attempts <= max_retries => {
                    error!("Operation failed (attempt {}): {:?}", attempts, e);
                    metrics::counter!(concat!($name, ".retry"), 1);
                    
                    // Exponential backoff
                    let backoff = Duration::from_millis(100 * 2u64.pow(attempts - 1));
                    tokio::time::sleep(backoff).await;
                    continue;
                }
                Ok(Err(e)) => {
                    metrics::counter!(concat!($name, ".failure"), 1);
                    break Err(e);
                }
                Err(_) => {
                    metrics::counter!(concat!($name, ".timeout"), 1);
                    break Err("Operation timed out".into());
                }
            }
        }
    }};
}

/// Macro for generating error types with context
#[macro_export]
macro_rules! define_errors {
    (
        $(
            $(#[$meta:meta])*
            $name:ident {
                $(
                    $(#[$variant_meta:meta])*
                    $variant:ident $(($($field:ty),*))? => $message:expr
                ),* $(,)?
            }
        )*
    ) => {
        $(
            $(#[$meta])*
            #[derive(Debug, thiserror::Error)]
            pub enum $name {
                $(
                    $(#[$variant_meta])*
                    #[error($message)]
                    $variant $(($($field),*))?,
                )*
                
                #[error("Internal error: {0}")]
                Internal(String),
                
                #[error(transparent)]
                Other(#[from] anyhow::Error),
            }
            
            impl $name {
                pub fn internal(msg: impl Into<String>) -> Self {
                    Self::Internal(msg.into())
                }
            }
        )*
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_macro() {
        define_messages! {
            TestMessage {
                content: String,
                priority: u8,
            }
        }

        let msg = TestMessage::new("Hello".to_string(), 5);
        assert_eq!(msg.content, "Hello");
        assert_eq!(msg.priority, 5);

        let msg2 = TestMessage::builder()
            .content("World".to_string())
            .priority(10)
            .build()
            .unwrap();
        assert_eq!(msg2.content, "World");
        assert_eq!(msg2.priority, 10);
    }
}