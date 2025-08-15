/// Universal generic handler for all modules
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Generic operation trait
#[async_trait]
pub trait Operation: Send + Sync {
    type Input;
    type Output;
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output>;
}

/// Generic state manager
pub struct StateManager<T> {
    state: Arc<RwLock<T>>,
}

impl<T: Default + Send + Sync> StateManager<T> {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(T::default())),
        }
    }
    
    pub async fn update<F, R>(&self, updater: F) -> Result<R>
    where
        F: FnOnce(&mut T) -> Result<R> + Send,
        R: Send,
    {
        let mut state = self.state.write().await;
        updater(&mut *state)
    }
    
    pub async fn read<F, R>(&self, reader: F) -> Result<R>
    where
        F: FnOnce(&T) -> Result<R> + Send,
        R: Send,
    {
        let state = self.state.read().await;
        reader(&*state)
    }
}

/// Generic list manager
pub struct ListManager<T> {
    items: Arc<RwLock<Vec<T>>>,
}

impl<T: Clone + Send + Sync> ListManager<T> {
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn add(&self, item: T) -> Result<()> {
        let mut items = self.items.write().await;
        items.push(item);
        Ok(())
    }
    
    pub async fn remove<F>(&self, predicate: F) -> Result<bool>
    where
        F: Fn(&T) -> bool,
    {
        let mut items = self.items.write().await;
        let len_before = items.len();
        items.retain(|item| !predicate(item));
        Ok(len_before != items.len())
    }
    
    pub async fn list(&self) -> Result<Vec<T>> {
        let items = self.items.read().await;
        Ok(items.clone())
    }
    
    pub async fn find<F>(&self, predicate: F) -> Result<Option<T>>
    where
        F: Fn(&T) -> bool,
    {
        let items = self.items.read().await;
        Ok(items.iter().find(|item| predicate(item)).cloned())
    }
}

/// Generic message processor
pub struct MessageProcessor<M, R> {
    handlers: Arc<RwLock<Vec<Box<dyn MessageHandler<M, R> + Send + Sync>>>>,
}

#[async_trait]
pub trait MessageHandler<M, R>: Send + Sync {
    async fn handle(&self, message: &M) -> Option<R>;
}

impl<M: Send + Sync, R: Send + Sync> MessageProcessor<M, R> {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn register(&self, handler: Box<dyn MessageHandler<M, R> + Send + Sync>) {
        let mut handlers = self.handlers.write().await;
        handlers.push(handler);
    }
    
    pub async fn process(&self, message: M) -> Vec<R> {
        let handlers = self.handlers.read().await;
        let mut results = Vec::new();
        
        for handler in handlers.iter() {
            if let Some(result) = handler.handle(&message).await {
                results.push(result);
            }
        }
        
        results
    }
}

/// Generic event system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event<T> {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: T,
}

pub struct EventBus<T> {
    subscribers: Arc<RwLock<Vec<Arc<dyn EventSubscriber<T> + Send + Sync>>>>,
}

#[async_trait]
pub trait EventSubscriber<T>: Send + Sync {
    async fn on_event(&self, event: &Event<T>);
}

impl<T: Clone + Send + Sync> EventBus<T> {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn subscribe(&self, subscriber: Arc<dyn EventSubscriber<T> + Send + Sync>) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.push(subscriber);
    }
    
    pub async fn publish(&self, data: T) {
        let event = Event {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            data,
        };
        
        let subscribers = self.subscribers.read().await;
        for subscriber in subscribers.iter() {
            subscriber.on_event(&event).await;
        }
    }
}

/// Generic resource pool
pub struct ResourcePool<T> {
    resources: Arc<RwLock<Vec<T>>>,
    max_size: usize,
}

impl<T: Clone + Send + Sync> ResourcePool<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            resources: Arc::new(RwLock::new(Vec::new())),
            max_size,
        }
    }
    
    pub async fn acquire(&self) -> Option<T> {
        let mut resources = self.resources.write().await;
        resources.pop()
    }
    
    pub async fn release(&self, resource: T) -> Result<()> {
        let mut resources = self.resources.write().await;
        if resources.len() < self.max_size {
            resources.push(resource);
        }
        Ok(())
    }
    
    pub async fn size(&self) -> usize {
        let resources = self.resources.read().await;
        resources.len()
    }
}