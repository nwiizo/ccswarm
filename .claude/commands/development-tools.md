# 開発ツール一覧

## 基本的な開発ツール

### Cargo (Rust)
```bash
# ビルド
cargo build
cargo build --release

# テスト
cargo test
cargo test -- --nocapture  # 出力を表示
cargo test --test integration_tests  # 統合テストのみ

# コード品質
cargo fmt  # フォーマット
cargo clippy -- -D warnings  # Lintチェック
cargo clean  # ビルドキャッシュをクリア

# 依存関係
cargo tree  # 依存関係ツリーを表示
cargo update  # 依存関係を更新
```

### Git
```bash
# ブランチ管理
git checkout -b feature/new-feature
git branch -d feature/old-feature

# コミット
git add .
git commit -m "feat: 新機能の追加"

# リモート操作
git push origin feature/branch-name
git pull origin main

# タグ管理
git tag -a v0.3.0 -m "Release version 0.3.0"
git push origin v0.3.0
```

### GitHub CLI (gh)
```bash
# リリース作成
gh release create v0.3.0 \
  --title "v0.3.0 - AI Agent Concepts Integration" \
  --notes-file release-notes.md \
  --prerelease

# PRの作成
gh pr create --title "feat: AIエージェント概念の統合" \
  --body "PR本文"

# Issue管理
gh issue list
gh issue create --title "バグ: コンパイルエラー"
```

## 開発中に使用した特殊なコマンド

### テストの一時的な無効化
```bash
# 失敗するテストにignore属性を追加
# src/test_file.rs内で
#[ignore]
#[test]
fn test_that_fails() { ... }
```

### 例ファイルの無効化
```bash
# .gitignoreに追加
examples/*.disabled

# ファイル名を変更
mv examples/failing_example.rs examples/failing_example.rs.disabled
```

### 機能フラグの使用
```bash
# Cargo.tomlに追加
[features]
default = []
container = ["bollard", "tempfile"]

# 条件付きコンパイル
#[cfg(feature = "container")]
mod container;
```

### 環境変数
```bash
# Rust開発
export RUST_LOG=debug  # デバッグログ
export RUST_BACKTRACE=1  # バックトレース表示

# API使用時
export ANTHROPIC_API_KEY="sk-..."
```

## エディタ/IDE設定

### VS Code拡張機能
- rust-analyzer: Rust言語サポート
- Even Better TOML: Cargo.toml編集
- crates: 依存関係バージョン管理

### 推奨設定 (.vscode/settings.json)
```json
{
  "rust-analyzer.cargo.features": ["container"],
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true
}
```

## デバッグツール

### ログ出力
```rust
// コード内でのログ
tracing::info!("情報ログ: {}", variable);
tracing::debug!("デバッグログ");
tracing::warn!("警告: {}", message);
tracing::error!("エラー: {}", error);
```

### テストのデバッグ
```bash
# 特定のテストを実行
cargo test test_name -- --exact --nocapture

# テストのバックトレース付き実行
RUST_BACKTRACE=1 cargo test
```

## CI/CD関連

### GitHub Actions設定
```yaml
# .github/workflows/ci.yml
name: CI
on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - run: cargo fmt -- --check
    - run: cargo clippy -- -D warnings
    - run: cargo test
```

## トラブルシューティング

### よくある問題と解決方法

1. **コンパイルエラー**
   ```bash
   cargo clean
   cargo build
   ```

2. **依存関係の競合**
   ```bash
   cargo update
   cargo tree --duplicates
   ```

3. **テストの失敗**
   ```bash
   # 環境をクリーンにして再実行
   cargo clean
   cargo test -- --test-threads=1
   ```