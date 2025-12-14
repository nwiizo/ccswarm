//! Embedding generation for vector similarity search

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A vector embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    /// The vector values
    pub vector: Vec<f32>,
    /// Model used to generate this embedding
    pub model: String,
    /// Dimension of the embedding
    pub dimension: usize,
}

impl Embedding {
    /// Create a new embedding
    pub fn new(vector: Vec<f32>, model: impl Into<String>) -> Self {
        let dimension = vector.len();
        Self {
            vector,
            model: model.into(),
            dimension,
        }
    }

    /// Calculate cosine similarity with another embedding
    pub fn similarity(&self, other: &Embedding) -> f32 {
        if self.vector.len() != other.vector.len() || self.vector.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = self
            .vector
            .iter()
            .zip(other.vector.iter())
            .map(|(a, b)| a * b)
            .sum();

        let norm_self: f32 = self.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_other: f32 = other.vector.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_self == 0.0 || norm_other == 0.0 {
            return 0.0;
        }

        dot_product / (norm_self * norm_other)
    }

    /// Normalize the vector
    pub fn normalize(&mut self) {
        let norm: f32 = self.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut self.vector {
                *v /= norm;
            }
        }
    }

    /// Get normalized copy
    pub fn normalized(&self) -> Self {
        let mut copy = self.clone();
        copy.normalize();
        copy
    }
}

/// Embedding model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingModel {
    /// Model name/ID
    pub name: String,
    /// Provider (openai, cohere, local, etc.)
    pub provider: String,
    /// Dimension of embeddings
    pub dimension: usize,
    /// Maximum input tokens
    pub max_tokens: usize,
}

impl EmbeddingModel {
    /// OpenAI text-embedding-3-small
    pub fn openai_small() -> Self {
        Self {
            name: "text-embedding-3-small".to_string(),
            provider: "openai".to_string(),
            dimension: 1536,
            max_tokens: 8191,
        }
    }

    /// OpenAI text-embedding-3-large
    pub fn openai_large() -> Self {
        Self {
            name: "text-embedding-3-large".to_string(),
            provider: "openai".to_string(),
            dimension: 3072,
            max_tokens: 8191,
        }
    }

    /// Cohere embed-english-v3.0
    pub fn cohere_english() -> Self {
        Self {
            name: "embed-english-v3.0".to_string(),
            provider: "cohere".to_string(),
            dimension: 1024,
            max_tokens: 512,
        }
    }

    /// Mock model for testing
    pub fn mock() -> Self {
        Self {
            name: "mock".to_string(),
            provider: "mock".to_string(),
            dimension: 384,
            max_tokens: 512,
        }
    }
}

/// Trait for embedding providers
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Get the model being used
    fn model(&self) -> &EmbeddingModel;

    /// Generate embedding for text
    async fn embed(&self, text: &str) -> Result<Embedding, String>;

    /// Generate embeddings for multiple texts
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>, String>;
}

/// Mock embedder for testing
pub struct MockEmbedder {
    model: EmbeddingModel,
}

impl MockEmbedder {
    /// Create a new mock embedder
    pub fn new() -> Self {
        Self {
            model: EmbeddingModel::mock(),
        }
    }

    /// Generate a deterministic embedding from text
    fn text_to_embedding(&self, text: &str) -> Vec<f32> {
        // Create a simple hash-based embedding for testing
        let mut embedding = vec![0.0f32; self.model.dimension];

        for (i, byte) in text.bytes().enumerate() {
            let idx = i % self.model.dimension;
            embedding[idx] += (byte as f32) / 255.0;
        }

        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut embedding {
                *v /= norm;
            }
        }

        embedding
    }
}

impl Default for MockEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbedder {
    fn model(&self) -> &EmbeddingModel {
        &self.model
    }

    async fn embed(&self, text: &str) -> Result<Embedding, String> {
        let vector = self.text_to_embedding(text);
        Ok(Embedding::new(vector, &self.model.name))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>, String> {
        let mut embeddings = Vec::with_capacity(texts.len());
        for text in texts {
            embeddings.push(self.embed(text).await?);
        }
        Ok(embeddings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_creation() {
        let embedding = Embedding::new(vec![0.1, 0.2, 0.3], "test-model");
        assert_eq!(embedding.dimension, 3);
        assert_eq!(embedding.model, "test-model");
    }

    #[test]
    fn test_embedding_similarity() {
        let e1 = Embedding::new(vec![1.0, 0.0, 0.0], "test");
        let e2 = Embedding::new(vec![1.0, 0.0, 0.0], "test");
        let e3 = Embedding::new(vec![0.0, 1.0, 0.0], "test");

        // Same vectors should have similarity 1.0
        assert!((e1.similarity(&e2) - 1.0).abs() < 0.001);

        // Orthogonal vectors should have similarity 0.0
        assert!((e1.similarity(&e3) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_embedding_normalize() {
        let mut embedding = Embedding::new(vec![3.0, 4.0], "test");
        embedding.normalize();

        // After normalization, length should be 1
        let length: f32 = embedding.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((length - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_embedding_models() {
        let small = EmbeddingModel::openai_small();
        assert_eq!(small.dimension, 1536);

        let large = EmbeddingModel::openai_large();
        assert_eq!(large.dimension, 3072);
    }

    #[tokio::test]
    async fn test_mock_embedder() {
        let embedder = MockEmbedder::new();

        let e1 = embedder.embed("hello world").await.unwrap();
        let e2 = embedder.embed("hello world").await.unwrap();
        let e3 = embedder.embed("goodbye world").await.unwrap();

        // Same text should produce same embedding
        assert!((e1.similarity(&e2) - 1.0).abs() < 0.001);

        // Different text should produce different embedding
        assert!(e1.similarity(&e3) < 1.0);
    }

    #[tokio::test]
    async fn test_mock_embedder_batch() {
        let embedder = MockEmbedder::new();

        let texts = vec!["hello".to_string(), "world".to_string()];
        let embeddings = embedder.embed_batch(&texts).await.unwrap();

        assert_eq!(embeddings.len(), 2);
    }

    #[test]
    fn test_similar_texts_have_similar_embeddings() {
        let embedder = MockEmbedder::new();

        let e1_vec = embedder.text_to_embedding("the quick brown fox");
        let e2_vec = embedder.text_to_embedding("the quick brown dog");
        let e3_vec = embedder.text_to_embedding("completely different text");

        let e1 = Embedding::new(e1_vec, "mock");
        let e2 = Embedding::new(e2_vec, "mock");
        let e3 = Embedding::new(e3_vec, "mock");

        // Similar texts should have higher similarity than different texts
        assert!(e1.similarity(&e2) > e1.similarity(&e3));
    }
}
