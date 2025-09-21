//! Symbol indexing system for fast code navigation
//!
//! Maintains an index of all symbols in the codebase for efficient lookup

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::RwLock;
use walkdir::WalkDir;

use super::analyzer::{Symbol, SymbolKind};
use super::{SemanticError, SemanticResult};

/// Symbol index for fast lookup
pub struct SymbolIndex {
    /// Symbol storage by path
    symbols_by_path: RwLock<HashMap<String, Symbol>>,

    /// Symbol storage by file
    symbols_by_file: RwLock<HashMap<PathBuf, Vec<String>>>,

    /// Symbol storage by name
    symbols_by_name: RwLock<HashMap<String, Vec<String>>>,

    /// Symbol storage by kind
    symbols_by_kind: RwLock<HashMap<SymbolKind, Vec<String>>>,

    /// Dependency graph
    dependency_graph: RwLock<HashMap<String, HashSet<String>>>,
}

impl SymbolIndex {
    /// Create a new symbol index
    pub async fn new() -> SemanticResult<Self> {
        Ok(Self {
            symbols_by_path: RwLock::new(HashMap::new()),
            symbols_by_file: RwLock::new(HashMap::new()),
            symbols_by_name: RwLock::new(HashMap::new()),
            symbols_by_kind: RwLock::new(HashMap::new()),
            dependency_graph: RwLock::new(HashMap::new()),
        })
    }

    /// Index the entire codebase
    pub async fn index_codebase(&self) -> SemanticResult<()> {
        log::info!("Starting codebase indexing...");

        // Clear existing index
        self.clear().await?;

        // Walk through source directories
        let source_dirs = vec!["crates/ccswarm/src", "crates/ai-session/src"];

        for dir in source_dirs {
            if Path::new(dir).exists() {
                self.index_directory(dir).await?;
            }
        }

        log::info!("Codebase indexing complete");
        Ok(())
    }

    /// Index a specific directory
    pub async fn index_directory(&self, dir: &str) -> SemanticResult<()> {
        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Only index Rust files for now
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                self.index_file(path).await?;
            }
        }

        Ok(())
    }

    /// Index a specific file
    pub async fn index_file(&self, path: &Path) -> SemanticResult<()> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| SemanticError::IndexError(format!("Failed to read file: {}", e)))?;

        // Parse and extract symbols (simplified for now)
        let symbols = self.extract_symbols_from_rust(&content, path)?;

        // Add symbols to index
        for symbol in symbols {
            self.add_symbol(symbol).await?;
        }

        Ok(())
    }

    /// Extract symbols from Rust code (simplified implementation)
    fn extract_symbols_from_rust(
        &self,
        content: &str,
        file_path: &Path,
    ) -> SemanticResult<Vec<Symbol>> {
        let mut symbols = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Extract functions
            if trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ") {
                if let Some(name) = self.extract_function_name(trimmed) {
                    symbols.push(Symbol {
                        name: name.clone(),
                        path: format!("{}::{}", file_path.display(), name),
                        kind: SymbolKind::Function,
                        file_path: file_path.to_string_lossy().to_string(),
                        line: line_num + 1,
                        body: None,
                        references: Vec::new(),
                        metadata: HashMap::new(),
                    });
                }
            }

            // Extract structs
            if trimmed.starts_with("pub struct ") || trimmed.starts_with("struct ") {
                if let Some(name) = self.extract_struct_name(trimmed) {
                    symbols.push(Symbol {
                        name: name.clone(),
                        path: format!("{}::{}", file_path.display(), name),
                        kind: SymbolKind::Struct,
                        file_path: file_path.to_string_lossy().to_string(),
                        line: line_num + 1,
                        body: None,
                        references: Vec::new(),
                        metadata: HashMap::new(),
                    });
                }
            }

            // Extract enums
            if trimmed.starts_with("pub enum ") || trimmed.starts_with("enum ") {
                if let Some(name) = self.extract_enum_name(trimmed) {
                    symbols.push(Symbol {
                        name: name.clone(),
                        path: format!("{}::{}", file_path.display(), name),
                        kind: SymbolKind::Enum,
                        file_path: file_path.to_string_lossy().to_string(),
                        line: line_num + 1,
                        body: None,
                        references: Vec::new(),
                        metadata: HashMap::new(),
                    });
                }
            }

            // Extract traits
            if trimmed.starts_with("pub trait ") || trimmed.starts_with("trait ") {
                if let Some(name) = self.extract_trait_name(trimmed) {
                    symbols.push(Symbol {
                        name: name.clone(),
                        path: format!("{}::{}", file_path.display(), name),
                        kind: SymbolKind::Trait,
                        file_path: file_path.to_string_lossy().to_string(),
                        line: line_num + 1,
                        body: None,
                        references: Vec::new(),
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        Ok(symbols)
    }

    /// Extract function name from declaration
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            if *part == "fn" && i + 1 < parts.len() {
                let name_part = parts[i + 1];
                if let Some(paren_pos) = name_part.find('(') {
                    return Some(name_part[..paren_pos].to_string());
                } else {
                    return Some(name_part.to_string());
                }
            }
        }
        None
    }

    /// Extract struct name from declaration
    fn extract_struct_name(&self, line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            if *part == "struct" && i + 1 < parts.len() {
                let name_part = parts[i + 1];
                if let Some(angle_pos) = name_part.find('<') {
                    return Some(name_part[..angle_pos].to_string());
                } else if let Some(curly_pos) = name_part.find('{') {
                    return Some(name_part[..curly_pos].to_string());
                } else {
                    return Some(name_part.to_string());
                }
            }
        }
        None
    }

    /// Extract enum name from declaration
    fn extract_enum_name(&self, line: &str) -> Option<String> {
        self.extract_struct_name(&line.replace("enum", "struct"))
    }

    /// Extract trait name from declaration
    fn extract_trait_name(&self, line: &str) -> Option<String> {
        self.extract_struct_name(&line.replace("trait", "struct"))
    }

    /// Add a symbol to the index
    pub async fn add_symbol(&self, symbol: Symbol) -> SemanticResult<()> {
        let path = symbol.path.clone();
        let name = symbol.name.clone();
        let kind = symbol.kind.clone();
        let file_path = PathBuf::from(&symbol.file_path);

        // Add to main index
        {
            let mut symbols = self.symbols_by_path.write().await;
            symbols.insert(path.clone(), symbol);
        }

        // Add to file index
        {
            let mut by_file = self.symbols_by_file.write().await;
            by_file.entry(file_path).or_default().push(path.clone());
        }

        // Add to name index
        {
            let mut by_name = self.symbols_by_name.write().await;
            by_name.entry(name).or_default().push(path.clone());
        }

        // Add to kind index
        {
            let mut by_kind = self.symbols_by_kind.write().await;
            by_kind.entry(kind).or_default().push(path.clone());
        }

        Ok(())
    }

    /// Find symbols by name
    pub async fn find_by_name(&self, name: &str) -> SemanticResult<Vec<Symbol>> {
        let by_name = self.symbols_by_name.read().await;
        let symbols_by_path = self.symbols_by_path.read().await;

        if let Some(paths) = by_name.get(name) {
            let symbols: Vec<Symbol> = paths
                .iter()
                .filter_map(|path| symbols_by_path.get(path))
                .cloned()
                .collect();
            Ok(symbols)
        } else {
            Ok(Vec::new())
        }
    }

    /// Find symbols by kind
    pub async fn find_by_kind(&self, kind: SymbolKind) -> SemanticResult<Vec<Symbol>> {
        let by_kind = self.symbols_by_kind.read().await;
        let symbols_by_path = self.symbols_by_path.read().await;

        if let Some(paths) = by_kind.get(&kind) {
            let symbols: Vec<Symbol> = paths
                .iter()
                .filter_map(|path| symbols_by_path.get(path))
                .cloned()
                .collect();
            Ok(symbols)
        } else {
            Ok(Vec::new())
        }
    }

    /// Find symbols in a file
    pub async fn find_in_file(&self, file_path: &Path) -> SemanticResult<Vec<Symbol>> {
        let by_file = self.symbols_by_file.read().await;
        let symbols_by_path = self.symbols_by_path.read().await;

        if let Some(paths) = by_file.get(file_path) {
            let symbols: Vec<Symbol> = paths
                .iter()
                .filter_map(|path| symbols_by_path.get(path))
                .cloned()
                .collect();
            Ok(symbols)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get all symbols
    pub async fn get_all_symbols(&self) -> SemanticResult<Vec<Symbol>> {
        let symbols = self.symbols_by_path.read().await;
        Ok(symbols.values().cloned().collect())
    }

    /// Clear the index
    pub async fn clear(&self) -> SemanticResult<()> {
        self.symbols_by_path.write().await.clear();
        self.symbols_by_file.write().await.clear();
        self.symbols_by_name.write().await.clear();
        self.symbols_by_kind.write().await.clear();
        self.dependency_graph.write().await.clear();
        Ok(())
    }

    /// Add a dependency relationship
    pub async fn add_dependency(&self, from: &str, to: &str) -> SemanticResult<()> {
        let mut graph = self.dependency_graph.write().await;
        graph
            .entry(from.to_string())
            .or_default()
            .insert(to.to_string());
        Ok(())
    }

    /// Get dependencies of a symbol
    pub async fn get_dependencies(&self, symbol_path: &str) -> SemanticResult<Vec<String>> {
        let graph = self.dependency_graph.read().await;
        if let Some(deps) = graph.get(symbol_path) {
            Ok(deps.iter().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }

    /// Get dependents of a symbol (reverse dependencies)
    pub async fn get_dependents(&self, symbol_path: &str) -> SemanticResult<Vec<String>> {
        let graph = self.dependency_graph.read().await;
        let dependents: Vec<String> = graph
            .iter()
            .filter_map(|(from, to_set)| {
                if to_set.contains(symbol_path) {
                    Some(from.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(dependents)
    }
}
