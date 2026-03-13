## プロジェクト目標

複数のプログラミング言語にわたる関数と型のコード類似性を計算する。

## プロジェクト構成

### クレート構成ポリシー

- **crates/core (similarity-ts-core)**: 言語非依存のコア機能
  - AST比較アルゴリズム (APTED, TSED)
  - 共通パーサーインターフェース (`LanguageParser` trait)
  - CLI共通ユーティリティ (`cli_parallel`, `cli_output`, `cli_file_utils`)

- **crates/similarity-ts**: TypeScript/JavaScript専用CLI
  - oxc_parser を使用した高速パース
  - 型システムの類似性検出 (type_comparator, type_extractor)
  - JSX/TSX サポート

- **crates/similarity-py**: Python専用CLI
  - tree-sitter-python を使用
  - Python固有の構文サポート
  - クラス・メソッドの検出

### 多言語サポートポリシー

1. **言語ごとに独立したCLIパッケージを提供**
   - `similarity-ts`: TypeScript/JavaScript専用
   - `similarity-py`: Python専用
   - 将来: `similarity-rs` (Rust), `similarity-go` (Go) など

2. **言語固有機能は各パッケージに実装**
   - TypeScript: 型システム、インターフェース、ジェネリクス、JSX
   - Python: デコレータ、内包表記、インデント構造
   - 各言語の特性に最適化した検出パターン

3. **共通機能はcoreに集約**
   - AST比較アルゴリズム
   - 並列処理フレームワーク
   - ファイル操作ユーティリティ
   - 将来のクロス言語比較の基盤

## 開発スタック

### Rust (メイン)
- cargo (workspace構成)
- clap (CLIフレームワーク)
- oxc_parser (TypeScript/JavaScriptパーサー - 高速)
- tree-sitter (Python, その他の言語 - 汎用的だが約10倍遅い)
- rayon (並列処理)

## コーディングルール

### Rust
- 標準的なRustの規約に従う
- lintにはclippyを使用する
- テストは `cargo test` で実行する
- pushする前には .github/workflows/rust.yaml 相当の確認のテストを実行して確認

## ディレクトリ構成

```
crates/              # Rust実装 (メイン)
  core/              # 言語非依存のコアロジック
  similarity-ts/     # TypeScript/JavaScript CLI
  similarity-py/     # Python CLI
examples/            # サンプルファイル
  mixed_language_project/  # 多言語サンプル
```

## 機能

### 共通機能 (全言語)

- ASTベースの比較による関数類似性検出
- 類似度閾値の設定
- クロスファイル分析のサポート
- VSCode互換の出力形式
- 並列処理によるパフォーマンス向上

### TypeScript/JavaScript固有

- 型の類似性検出 (インターフェース、型エイリアス、型リテラル)
- クラスの類似性検出 (プロパティ、メソッド、継承)
- JSX/TSXサポート
- ES6+構文サポート (アロー関数、クラスなど)
- oxc_parserによる高速パース

### Python固有

- クラスとメソッドの検出
- デコレータサポート
- Python 3.x構文サポート
- インデントベースの構造分析

## 将来の言語拡張

新しい言語を追加する場合:

1. `crates/similarity-{lang}` ディレクトリを作成
2. `LanguageParser` trait を実装
3. 言語固有の機能を実装
4. 統合テストを追加

## 実装上の重要な注意事項

### 類似度計算には必ず calculate_tsed を使用する

**重要**: AST の類似度を計算する際は、必ず `similarity_core::tsed::calculate_tsed` 関数を使用すること。

**間違った実装例**:
```rust
// ❌ 直接計算してはいけない
let dist = compute_edit_distance(&tree1, &tree2, &options.apted_options);
let size1 = tree1.get_subtree_size();
let size2 = tree2.get_subtree_size();
let max_size = size1.max(size2) as f64;
let similarity = if max_size > 0.0 { 1.0 - (dist / max_size) } else { 1.0 };
```

**正しい実装例**:
```rust
// ✅ calculate_tsed を使用する
let similarity = similarity_core::tsed::calculate_tsed(&tree1, &tree2, options);
```

**理由**:
- `calculate_tsed` は `size_penalty` などの重要なオプションを適用する
- サイズが大きく異なる関数間の false positive を防ぐ
- 小さい関数に対する適切なペナルティを適用する

この間違いは特に新しい言語サポートを追加する際に発生しやすいため、必ず確認すること。

```