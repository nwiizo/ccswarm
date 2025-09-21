/// Task execution pipeline using Rust's iterator patterns
///
/// This module implements efficient task processing using
/// zero-cost abstractions and functional programming patterns.

use crate::agent::{Task, Priority, TaskResult};
use crate::error::Result;
use std::collections::HashMap;

/// Task pipeline for efficient batch processing
pub struct TaskPipeline {
    tasks: Vec<Task>,
}

impl TaskPipeline {
    pub fn new(tasks: Vec<Task>) -> Self {
        Self { tasks }
    }

    /// Filter tasks by priority using iterator adapters
    pub fn filter_priority(self, priority: Priority) -> impl Iterator<Item = Task> {
        self.tasks
            .into_iter()
            .filter(move |task| task.priority == priority)
    }

    /// Map tasks to assignments with agents
    pub fn assign_to_agents<'a>(
        self,
        agents: &'a [String],
    ) -> impl Iterator<Item = (Task, &'a str)> + 'a {
        self.tasks
            .into_iter()
            .zip(agents.iter().cycle())
            .map(|(task, agent)| (task, agent.as_str()))
    }

    /// Process tasks in parallel batches
    pub async fn process_batch<F, Fut>(self, batch_size: usize, processor: F) -> Vec<Result<TaskResult>>
    where
        F: Fn(Task) -> Fut + Clone,
        Fut: std::future::Future<Output = Result<TaskResult>>,
    {
        use futures::stream::{self, StreamExt};

        stream::iter(self.tasks)
            .chunks(batch_size)
            .flat_map(|batch| {
                stream::iter(batch)
                    .map(processor.clone())
                    .buffer_unordered(batch_size)
            })
            .collect()
            .await
    }

    /// Chain multiple transformations efficiently
    pub fn transform(self) -> TaskTransformer {
        TaskTransformer::new(self.tasks)
    }
}

/// Fluent API for task transformations
pub struct TaskTransformer {
    tasks: Vec<Task>,
}

impl TaskTransformer {
    fn new(tasks: Vec<Task>) -> Self {
        Self { tasks }
    }

    /// Sort by priority (in-place for efficiency)
    pub fn sort_by_priority(mut self) -> Self {
        self.tasks.sort_by_key(|t| t.priority);
        self
    }

    /// Deduplicate tasks by ID
    pub fn deduplicate(mut self) -> Self {
        use std::collections::HashSet;

        let mut seen = HashSet::new();
        self.tasks.retain(|task| seen.insert(task.id.clone()));
        self
    }

    /// Group by metadata field
    pub fn group_by<K, V>(self, key_fn: impl Fn(&Task) -> K) -> HashMap<K, Vec<Task>>
    where
        K: Eq + std::hash::Hash,
    {
        let mut groups = HashMap::new();

        for task in self.tasks {
            let key = key_fn(&task);
            groups.entry(key).or_insert_with(Vec::new).push(task);
        }

        groups
    }

    /// Apply a transformation function
    pub fn map<F, R>(self, f: F) -> impl Iterator<Item = R>
    where
        F: Fn(Task) -> R,
    {
        self.tasks.into_iter().map(f)
    }

    /// Collect results
    pub fn collect(self) -> Vec<Task> {
        self.tasks
    }
}

/// Efficient task aggregation using iterators
pub struct TaskAggregator;

impl TaskAggregator {
    /// Calculate statistics without allocating intermediate collections
    pub fn calculate_stats(tasks: impl Iterator<Item = Task>) -> TaskStats {
        let mut total = 0;
        let mut high_priority = 0;
        let mut medium_priority = 0;
        let mut low_priority = 0;

        for task in tasks {
            total += 1;
            match task.priority {
                Priority::High | Priority::Critical => high_priority += 1,
                Priority::Medium => medium_priority += 1,
                Priority::Low => low_priority += 1,
            }
        }

        TaskStats {
            total,
            high_priority,
            medium_priority,
            low_priority,
        }
    }

    /// Find tasks matching criteria without allocation
    pub fn find_matching<'a>(
        tasks: &'a [Task],
        predicate: impl Fn(&Task) -> bool + 'a,
    ) -> impl Iterator<Item = &'a Task> + 'a {
        tasks.iter().filter(move |task| predicate(task))
    }
}

#[derive(Debug, PartialEq)]
pub struct TaskStats {
    pub total: usize,
    pub high_priority: usize,
    pub medium_priority: usize,
    pub low_priority: usize,
}

/// Custom iterator for task pagination
pub struct TaskPaginator<I> {
    iter: I,
    page_size: usize,
}

impl<I> TaskPaginator<I>
where
    I: Iterator<Item = Task>,
{
    pub fn new(iter: I, page_size: usize) -> Self {
        Self { iter, page_size }
    }
}

impl<I> Iterator for TaskPaginator<I>
where
    I: Iterator<Item = Task>,
{
    type Item = Vec<Task>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut page = Vec::with_capacity(self.page_size);

        for _ in 0..self.page_size {
            match self.iter.next() {
                Some(task) => page.push(task),
                None => break,
            }
        }

        if page.is_empty() {
            None
        } else {
            Some(page)
        }
    }
}

/// Extension trait for task iterators
pub trait TaskIteratorExt: Iterator<Item = Task> + Sized {
    fn paginate(self, page_size: usize) -> TaskPaginator<Self> {
        TaskPaginator::new(self, page_size)
    }

    fn high_priority_only(self) -> impl Iterator<Item = Task> {
        self.filter(|task| task.priority == Priority::High)
    }

    fn with_metadata(self, key: String) -> impl Iterator<Item = Task> {
        self.filter(move |task| {
            task.metadata.as_ref().map_or(false, |m| m.get(&key).is_some())
        })
    }
}

impl<I: Iterator<Item = Task> + Sized> TaskIteratorExt for I {}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tasks() -> Vec<Task> {
        vec![
            Task {
                id: "1".to_string(),
                description: "Task 1".to_string(),
                priority: Priority::High,
                task_type: crate::agent::TaskType::Development,
                agent_id: None,
                metadata: None,
                tags: vec![],
                context: None,
            },
            Task {
                id: "2".to_string(),
                description: "Task 2".to_string(),
                priority: Priority::Medium,
                task_type: crate::agent::TaskType::Development,
                agent_id: None,
                metadata: None,
                tags: vec![],
                context: None,
            },
            Task {
                id: "3".to_string(),
                description: "Task 3".to_string(),
                priority: Priority::Low,
                task_type: crate::agent::TaskType::Development,
                agent_id: None,
                metadata: None,
                tags: vec![],
                context: None,
            },
        ]
    }

    #[test]
    fn test_pipeline_filter() {
        let tasks = create_test_tasks();
        let pipeline = TaskPipeline::new(tasks);

        let high_priority: Vec<_> = pipeline.filter_priority(Priority::High).collect();
        assert_eq!(high_priority.len(), 1);
        assert_eq!(high_priority[0].id, "1");
    }

    #[test]
    fn test_transformer_chain() {
        let tasks = create_test_tasks();
        let pipeline = TaskPipeline::new(tasks);

        let result = pipeline
            .transform()
            .sort_by_priority()
            .deduplicate()
            .collect();

        assert_eq!(result.len(), 3);
        // Tasks should be sorted: Low, Medium, High
        assert_eq!(result[0].priority, Priority::Low);
        assert_eq!(result[1].priority, Priority::Medium);
        assert_eq!(result[2].priority, Priority::High);
    }

    #[test]
    fn test_aggregator_stats() {
        let tasks = create_test_tasks();
        let stats = TaskAggregator::calculate_stats(tasks.into_iter());

        assert_eq!(stats.total, 3);
        assert_eq!(stats.high_priority, 1);
        assert_eq!(stats.medium_priority, 1);
        assert_eq!(stats.low_priority, 1);
    }

    #[test]
    fn test_pagination() {
        let tasks = create_test_tasks();
        let pages: Vec<_> = tasks.into_iter().paginate(2).collect();

        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].len(), 2);
        assert_eq!(pages[1].len(), 1);
    }
}