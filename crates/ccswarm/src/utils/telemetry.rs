use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Telemetry collection and metrics
pub struct Telemetry {
    metrics: Arc<RwLock<HashMap<String, Metric>>>,
}

#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub value: MetricValue,
    pub tags: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum MetricValue {
    Counter(i64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Summary(SummaryData),
}

#[derive(Debug, Clone)]
pub struct SummaryData {
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
}

impl Telemetry {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record_counter(&self, name: impl Into<String>, value: i64) {
        let mut metrics = self.metrics.write().await;
        let name = name.into();

        metrics.insert(
            name.clone(),
            Metric {
                name,
                value: MetricValue::Counter(value),
                tags: HashMap::new(),
                timestamp: chrono::Utc::now(),
            },
        );
    }

    pub async fn record_gauge(&self, name: impl Into<String>, value: f64) {
        let mut metrics = self.metrics.write().await;
        let name = name.into();

        metrics.insert(
            name.clone(),
            Metric {
                name,
                value: MetricValue::Gauge(value),
                tags: HashMap::new(),
                timestamp: chrono::Utc::now(),
            },
        );
    }

    pub async fn get_metrics(&self) -> HashMap<String, Metric> {
        self.metrics.read().await.clone()
    }
}

impl Default for Telemetry {
    fn default() -> Self {
        Self::new()
    }
}