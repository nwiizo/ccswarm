# /mutation-test - ミューテーションテスト実行

cargo-mutants を使用してミューテーションテストを実行し、テストの品質を検証する。

## 実行内容

1. cargo-mutants がインストールされているか確認
2. 指定されたクレートでミューテーションテストを実行
3. 結果を分析し、未検出のミュータント（生き残り）を報告

## コマンド

```bash
# インストール確認
cargo mutants --version || cargo install cargo-mutants

# ミュータント一覧（実行前の確認）
cargo mutants --list -p <crate-name>

# ミューテーションテスト実行
cargo mutants -p <crate-name> --timeout 120 -j 4

# 結果確認
cat mutants.out/caught.txt    # 検出されたミュータント
cat mutants.out/missed.txt    # 未検出のミュータント（要改善）
cat mutants.out/timeout.txt   # タイムアウトしたミュータント
```

## 結果の解釈

| 結果 | 意味 | 対応 |
|-----|------|------|
| caught | テストがミュータントを検出 | 良好、対応不要 |
| missed | テストがミュータントを検出できず | テスト追加が必要 |
| timeout | テスト実行がタイムアウト | タイムアウト値の調整を検討 |
| unviable | ビルドできないミュータント | 対応不要 |

## 重点的に確認すべき箇所

- `missed` のミュータントが多い関数はテストカバレッジが不足している
- ビジネスロジック（usecase/）の missed は優先的に対応する
- エラーハンドリング（error/）の missed も重要

## 出力例

```
317 mutants tested
- 280 caught (88%)
- 20 missed (6%)
- 10 timeout (3%)
- 7 unviable (2%)
```

## 参考

- [cargo-mutants ドキュメント](https://mutants.rs/)
- ミューテーションテストはテストの品質を測定する手法
- 「ミュータントを殺す」= テストがコード変更を検出できる

## 実行手順

ccswarm クレートに対してミューテーションテストを実行:

```bash
# 1. インストール確認
cargo mutants --version || cargo install cargo-mutants

# 2. ミュータント一覧確認（任意）
cargo mutants --list -p ccswarm | head -50

# 3. ミューテーションテスト実行（並列4、タイムアウト120秒）
cargo mutants -p ccswarm --timeout 120 -j 4

# 4. 結果サマリー表示
echo "=== Caught ===" && wc -l mutants.out/caught.txt
echo "=== Missed ===" && cat mutants.out/missed.txt
echo "=== Timeout ===" && wc -l mutants.out/timeout.txt
```
