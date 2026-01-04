# LocalDNS Pro

ローカル開発環境用のハイブリッドDNSサーバー

## 概要

LocalDNS Proは、ローカル開発環境でドメイン解決を柔軟に管理するためのDNSサーバーです。
SQLiteベースの動的レコード管理とReactによるWeb UIを提供します。

## 主な機能

- ✅ SQLiteによる動的なDNSレコード管理
- ✅ A / AAAA / CNAME レコードのサポート
- ✅ ワイルドカードドメインパターン対応
- ✅ レコードキャッシュによる高速応答
- ✅ React + TailwindCSS による Web UI
- ✅ クエリログのリアルタイム表示
- ✅ 非同期ログ記録による高いパフォーマンス

## システム要件

- **Rust**: 1.70以上
- **Node.js**: 18以上
- **OS**: Windows 10/11 または Linux

## セットアップ

### 1. 依存関係のインストール

```bash
# Rust依存関係
cargo build

# フロントエンド依存関係
cd web-ui
npm install
cd ..
```

### 2. 開発モード

**ワンコマンド起動 (推奨)**

PowerShellで以下のコマンドを実行:

```powershell
.\start-dev-simple.ps1
```

バックエンドとフロントエンドが別ウィンドウで自動起動します。

**個別起動**

バックエンド (Rust):
```powershell
cargo run
```

フロントエンド (React) - 別のPowerShellウィンドウで:
```powershell
cd web-ui
npm run dev
```

開発中は、Viteの開発サーバー (http://localhost:5173) でReact UIを編集できます。
バックエンドAPIは http://localhost:3000 で提供されます。

### 3. プロダクションビルド

```bash
# フロントエンドをビルド
cd web-ui
npm run build
cd ..

# Rustバイナリにフロントエンドを埋め込んでビルド
cargo build --release
```

### 4. 実行

```bash
# Windows (管理者権限で実行)
target/release/local-dns-pro.exe

# Linux (rootまたはCAP_NET_BIND_SERVICE付与)
sudo target/release/local-dns-pro
```

起動後、以下にアクセスできます:
- Web UI: http://localhost:3000

## 使い方

### レコードの追加

1. Web UIの「レコード管理」ページを開く
2. 「新規レコード追加」をクリック
3. 以下の情報を入力:
   - **ドメインパターン**: `app.local.test` または `%.local.test`
   - **レコードタイプ**: A, AAAA, CNAME
   - **コンテンツ**: IPアドレスまたはホスト名
   - **TTL**: 秒数 (デフォルト60)
4. 「作成」をクリック

### 設定の変更

1. Web UIの「設定」ページを開く
2. 各設定項目を編集:
   - **プライマリDNS**: 最初に問い合わせる上位DNSサーバー
   - **セカンダリDNS**: プライマリが失敗した場合のDNSサーバー
   - **タイムアウト**: 上位DNSへの問い合わせタイムアウト時間
   - **ログ保存期間**: この日数を超えたログは自動削除
3. 「保存」をクリック

## テスト

```bash
cargo test
```

## 詳細仕様

詳細な設計仕様は [DESIGN.md](./DESIGN.md) を参照してください。