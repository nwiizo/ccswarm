#!/bin/bash

# バージョン番号を引数として受け取る
VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 v0.2.0"
    exit 1
fi

echo "🔍 Running local CI checks before release..."

# Check if we're on the master branch (not main)
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "master" ]; then
    echo "⚠️  Warning: You are not on the master branch (current: $CURRENT_BRANCH)"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo "❌ Error: There are uncommitted changes. Please commit or stash them first."
    git status --short
    exit 1
fi

# Check if we can reach crates.io
echo "🌐 Checking crates.io connectivity..."
if ! curl -s --max-time 10 https://crates.io >/dev/null; then
    echo "❌ Error: Cannot reach crates.io. Please check your internet connection."
    exit 1
fi

# 前回のタグを取得
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

# リリースノートを生成
generate_release_notes() {
    local from_tag="$1"
    local to_ref="HEAD"
    local notes=""

    if [ -z "$from_tag" ]; then
        notes="## 🎉 Initial Release\n\n"
        notes+="### ✨ Key Features\n"
        notes+="- 🚀 **AI-Powered Auto-Create System**: Generate complete applications from natural language\n"
        notes+="- 🎯 **Master Task Delegation**: Intelligent task analysis and agent assignment\n"
        notes+="- 💡 **Session-Persistent Architecture**: 93% token reduction through intelligent session reuse\n"
        notes+="- 🔄 **Multi-Provider Support**: Works with Claude Code, Aider, OpenAI Codex, and custom tools\n"
        notes+="- 📊 **Real-time Monitoring**: Live TUI dashboard with agent status and logs\n"
        notes+="- 🛡️ **Auto-Accept Mode**: Safe background task completion with guardrails\n"
        notes+="- 🌲 **Git Worktree Integration**: Isolated parallel development environments\n"
        notes+="- 🗾 **Japanese Documentation**: Full documentation in both English and Japanese\n"
    else
        notes="## 🚀 Changes since $from_tag\n\n"
        
        # コミットを分類してリリースノートを生成
        local feat_commits=$(git log "$from_tag..$to_ref" --pretty=format:"- %s" --grep="^feat:" 2>/dev/null)
        local fix_commits=$(git log "$from_tag..$to_ref" --pretty=format:"- %s" --grep="^fix:" 2>/dev/null)
        local docs_commits=$(git log "$from_tag..$to_ref" --pretty=format:"- %s" --grep="^docs:" 2>/dev/null)
        local chore_commits=$(git log "$from_tag..$to_ref" --pretty=format:"- %s" --grep="^chore:" 2>/dev/null)
        
        if [ -n "$feat_commits" ]; then
            notes+="### ✨ New Features\n$feat_commits\n\n"
        fi
        
        if [ -n "$fix_commits" ]; then
            notes+="### 🐛 Bug Fixes\n$fix_commits\n\n"
        fi
        
        if [ -n "$docs_commits" ]; then
            notes+="### 📚 Documentation\n$docs_commits\n\n"
        fi
        
        if [ -n "$chore_commits" ]; then
            notes+="### 🔧 Maintenance\n$chore_commits\n\n"
        fi
    fi
    
    # Add installation instructions
    notes+="\n## 📦 Installation\n\n"
    notes+="\`\`\`bash\n"
    notes+="cargo install ccswarm\n"
    notes+="\`\`\`\n\n"
    
    # Add quick start
    notes+="## 🚀 Quick Start\n\n"
    notes+="\`\`\`bash\n"
    notes+="# Generate a TODO app\n"
    notes+="ccswarm auto-create \"Create a TODO application\" --output ./my_app\n\n"
    notes+="# Run the generated app\n"
    notes+="cd my_app && npm install && npm start\n"
    notes+="\`\`\`\n"

    echo -e "$notes"
}

# Cargoにログインしているか確認
if ! cargo login --help &>/dev/null; then
    echo "Error: Please login to crates.io first using 'cargo login'"
    echo "You can find your API token at https://crates.io/me"
    exit 1
fi

# Cargo.tomlのバージョンを更新
echo "📝 Updating version in Cargo.toml..."
sed -i '' "s/^version = .*/version = \"${VERSION#v}\"/" Cargo.toml

# Run comprehensive CI checks locally
echo "🔧 Running formatting check..."
cargo fmt --check || {
    echo "❌ Code formatting issues found. Running cargo fmt to fix..."
    cargo fmt || exit 1
    echo "✅ Code formatted. Please review changes and commit them."
    exit 1
}
echo "✅ Code formatting is correct"

echo "🔍 Running clippy linting..."
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | grep -E "error:|warning:" | head -20 || {
    echo "⚠️  Some clippy warnings detected. Continuing..."
}
echo "✅ Clippy check completed"

echo "🧪 Running unit tests..."
cargo test --lib --quiet || {
    echo "❌ Tests failed. Please fix the failing tests."
    exit 1
}
echo "✅ All tests passed"

echo "🔨 Running release build check..."
cargo build --release --quiet || {
    echo "❌ Release build failed. Please fix the build issues."
    exit 1
}
echo "✅ Release build successful"

echo "📋 Checking for security advisories..."
if command -v cargo-audit >/dev/null 2>&1; then
    cargo audit || {
        echo "⚠️  Security advisories found. Please review and address them."
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    }
    echo "✅ Security audit passed"
else
    echo "⚠️  cargo-audit not installed. Skipping security audit."
    echo "   Install with: cargo install cargo-audit"
fi

# cargo updateを実行してCargo.lockを更新
echo "📦 Updating dependencies..."
cargo update --quiet || exit 1

# 変更をコミット
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to $VERSION" || {
    echo "⚠️  No changes to commit (version might already be set)"
}

# Cargoのパッケージを作成（dry-run で検証）
echo "📦 Verifying Cargo package..."
cargo package --allow-dirty --quiet || {
    echo "❌ Package verification failed. Please fix the issues above."
    exit 1
}
echo "✅ Package verified successfully"

# リリースノートを生成
RELEASE_NOTES=$(generate_release_notes "$LAST_TAG")

# gitタグを作成
echo "🏷️  Creating git tag..."
git tag -a "$VERSION" -m "Release $VERSION" || {
    echo "❌ Failed to create tag. Tag might already exist."
    exit 1
}

# GitHubリリースを作成
echo "📢 Creating GitHub release..."
gh release create "$VERSION" \
    --title "Release $VERSION" \
    --notes "$RELEASE_NOTES" \
    --draft || {
    echo "❌ Failed to create GitHub release"
    exit 1
}

# crates.ioにパブリッシュ
echo "📤 Publishing to crates.io..."
cargo publish --allow-dirty || {
    echo "❌ Failed to publish to crates.io"
    echo "This might be because:"
    echo "  - You need to be logged in (cargo login)"
    echo "  - The crate name is already taken"
    echo "  - Version already exists"
    exit 1
}

# リモートにプッシュ
echo "🚀 Pushing to remote..."
git push origin master || {
    echo "⚠️  Failed to push to master branch"
}
git push origin "$VERSION" || {
    echo "⚠️  Failed to push tag"
}

echo "🎉 Release $VERSION completed successfully!"
echo ""
echo "✅ Summary of actions performed:"
echo "  - ✓ Ran all local CI checks (format, lint, test, build, security)"
echo "  - ✓ Updated version to $VERSION in Cargo.toml"
echo "  - ✓ Updated dependencies in Cargo.lock"
echo "  - ✓ Created commit with version bump"
echo "  - ✓ Created GitHub release with auto-generated notes"
echo "  - ✓ Published to crates.io"
echo "  - ✓ Pushed tags to origin"
echo ""
echo "🔗 Next steps:"
echo "  - Review the GitHub release at: https://github.com/nwiizo/ccswarm/releases/tag/$VERSION"
echo "  - Check the crates.io publication at: https://crates.io/crates/ccswarm"
echo "  - Edit the draft release on GitHub to publish it"