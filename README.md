# rs_har_analyzer

HARファイル（HTTP Archive）を解析してExcelファイルに出力するRust製のコマンドラインツールです。

## 機能

- HARファイルの読み込みと解析
- GET/POSTリクエストの抽出
- JSONペイロードの整形
- 解析結果のExcelファイル出力

## インストール方法

### cargo installを使用してインストール

```bash
cargo install --git https://github.com/yourusername/rs_har_analyzer
```

### ソースコードからビルド

```bash
git clone https://github.com/yourusername/rs_har_analyzer
cd rs_har_analyzer
cargo build --release
```

## 使用方法

### 基本的な使用法

```bash
rs_har_analyzer -i input.har -o output.xlsx
```

### オプション

- `-i, --input <FILE>`: 入力するHARファイルのパス（必須）
- `-o, --output <FILE>`: 出力するExcelファイルのパス（デフォルト: har_analysis.xlsx）
- `-v, --verbose`: 詳細ログを出力
- `-h, --help`: ヘルプメッセージを表示

### 使用例

```bash
# 基本的な使用
rs_har_analyzer -i my_session.har -o analysis_result.xlsx

# 詳細ログ付きで実行
rs_har_analyzer -i my_session.har -o analysis_result.xlsx -v
```

## 出力フォーマット

Excelファイルには以下の列が含まれます：

| 列名 | 説明 |
|------|------|
| 時刻 | リクエストの開始時刻 |
| 送信元IP | リクエストの送信元IP |
| 送信先IP | リクエストの送信先IP |
| メソッド | HTTPメソッド（GET/POST） |
| ステータスコード | HTTPステータスコード |
| リクエストURL | リクエストURL |
| リクエストペイロード | リクエストのJSONペイロード |
| レスポンスペイロード | レスポンスのJSONペイロード |

## 必要な環境

- Rust 1.70以上
- Windows 11（他のOSでも動作する可能性があります）

## ライセンス

MIT License

## 貢献

プルリクエストやイシューの報告を歓迎します。

## 変更履歴

### v0.1.0
- 初回リリース
- HARファイルの基本的な解析機能
- Excelファイルへの出力機能
