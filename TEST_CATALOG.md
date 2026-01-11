# テストカタログ

## 概要

### バックエンド（Rust）

| カテゴリ | テスト数 | 種類 |
|----------|----------|------|
| db::models | 3 | 単体テスト |
| db | 8 | 統合テスト |
| dns::cache | 3 | 統合テスト |
| dns::handler | 4 | 統合テスト |
| dns::resolver | 4 | 単体テスト |
| dns::upstream | 3 | 単体/統合テスト |
| logger::worker | 2 | 統合テスト |
| web::api | 15 | 統合テスト |
| **小計** | **42** | |

### フロントエンド（React + TypeScript）

| カテゴリ | テスト数 | 種類 |
|----------|----------|------|
| api | 16 | 単体テスト |
| App | 6 | 統合テスト |
| Dashboard | 8 | 統合テスト |
| Records | 11 | 統合テスト |
| Settings | 8 | 統合テスト |
| **小計** | **49** | |

### 全体合計: **91テスト**

---

## 実行方法

### バックエンド

```bash
# 全テスト実行
cargo test

# 特定モジュールのテスト
cargo test dns::cache
cargo test web::api

# 特定テスト実行
cargo test test_record_matches_exact

# テスト一覧表示
cargo test -- --list

# カバレッジ測定
cargo llvm-cov --summary-only
```

### フロントエンド

```bash
cd web-ui

# 全テスト実行（ウォッチモード）
npm test

# 全テスト実行（単発）
npm run test:run

# カバレッジ測定
npm run test:coverage
```

---

## カバレッジ

**測定日**: 2026-01-11

### バックエンド

| ファイル | カバレッジ |
|----------|-----------|
| dns/cache.rs | 97.52% |
| db/mod.rs | 96.71% |
| web/api.rs | 95.12% |
| db/models.rs | 92.98% |
| logger/worker.rs | 87.78% |
| dns/upstream.rs | 85.85% |
| dns/resolver.rs | 81.51% |
| dns/handler.rs | 54.55% |
| main.rs | 0.00% |
| web/router.rs | 0.00% |
| **合計** | **82.06%** |

### フロントエンド

| ファイル | カバレッジ |
|----------|-----------|
| App.tsx | 100.00% |
| api.ts | 100.00% |
| Dashboard.tsx | 100.00% |
| Settings.tsx | 93.10% |
| Records.tsx | 84.09% |
| ThemeToggle.tsx | 87.50% |
| ThemeProvider.tsx | 70.73% |
| **合計** | **86.33%** |

---

## テスト詳細

### 1. db::models（レコードモデル）

| テスト名 | 説明 | ファイル |
|----------|------|----------|
| `test_record_matches_exact` | 完全一致ドメインのマッチング検証 | src/db/models.rs:108 |
| `test_record_matches_wildcard` | ワイルドカード（`%`）パターンのマッチング検証 | src/db/models.rs:123 |
| `test_record_matches_inactive` | 非アクティブレコードがマッチしないことを検証 | src/db/models.rs:140 |

### 2. db（データベース操作）

| テスト名 | 説明 | ファイル |
|----------|------|----------|
| `test_create_and_get_record` | レコードの作成と取得 | src/db/mod.rs:296 |
| `test_update_record` | レコードの更新 | src/db/mod.rs:316 |
| `test_delete_record` | レコードの削除 | src/db/mod.rs:344 |
| `test_settings` | 設定値の取得・更新 | src/db/mod.rs:363 |
| `test_log_query_and_get_recent` | クエリログの記録と取得 | src/db/mod.rs:379 |
| `test_cleanup_old_logs` | 古いログのクリーンアップ | src/db/mod.rs:417 |
| `test_cleanup_old_logs_no_old_logs` | 削除対象がない場合のクリーンアップ | src/db/mod.rs:461 |

### 3. dns::cache（レコードキャッシュ）

| テスト名 | 説明 | ファイル |
|----------|------|----------|
| `test_cache_reload` | キャッシュの再読み込み機能 | src/dns/cache.rs:99 |
| `test_find_matching_record` | キャッシュからのレコード検索 | src/dns/cache.rs:120 |
| `test_multiple_records_priority` | 完全一致がワイルドカードより優先されることを検証 | src/dns/cache.rs:146 |

### 4. dns::handler（DNSハンドラ）

| テスト名 | 説明 | ファイル |
|----------|------|----------|
| `test_dns_handler_cache_hit` | キャッシュヒット時のDNS応答 | src/dns/handler.rs:160 |
| `test_dns_handler_with_upstream` | 上位DNS転送が有効な場合 | src/dns/handler.rs:184 |
| `test_dns_handler_without_upstream` | 上位DNS転送が無効な場合 | src/dns/handler.rs:199 |
| `test_dns_handler_clone` | ハンドラのクローン機能 | src/dns/handler.rs:211 |

### 5. dns::resolver（レコード構築）

| テスト名 | 説明 | ファイル |
|----------|------|----------|
| `test_build_a_record` | Aレコード（IPv4）の構築 | src/dns/resolver.rs:101 |
| `test_build_aaaa_record` | AAAAレコード（IPv6）の構築 | src/dns/resolver.rs:124 |
| `test_build_cname_record` | CNAMEレコードの構築 | src/dns/resolver.rs:145 |
| `test_build_invalid_a_record` | 不正なIPアドレスでのエラー処理 | src/dns/resolver.rs:171 |

### 6. dns::upstream（上位DNS転送）

| テスト名 | 説明 | ファイル |
|----------|------|----------|
| `test_upstream_config_new` | 上位DNS設定の初期化 | src/dns/upstream.rs:157 |
| `test_upstream_config_invalid_address` | 不正なアドレスでのエラー処理 | src/dns/upstream.rs:172 |
| `test_query_real` | 実際の上位DNS問い合わせ（google.com） | src/dns/upstream.rs:178 |

### 7. logger::worker（非同期ログ）

| テスト名 | 説明 | ファイル |
|----------|------|----------|
| `test_log_worker` | 単一ログメッセージの保存 | src/logger/worker.rs:122 |
| `test_log_worker_multiple_messages` | 複数ログメッセージの保存 | src/logger/worker.rs:145 |

### 8. web::api（Web API）

| テスト名 | 説明 | ファイル |
|----------|------|----------|
| `test_health_check` | ヘルスチェックエンドポイント | src/web/api.rs:268 |
| `test_get_records_empty` | 空のレコード一覧取得 | src/web/api.rs:290 |
| `test_create_and_get_record` | レコードの作成と取得 | src/web/api.rs:311 |
| `test_create_record_validation_empty_domain` | 空ドメインのバリデーション | src/web/api.rs:370 |
| `test_create_record_validation_invalid_ip` | 不正なIPアドレスのバリデーション | src/web/api.rs:396 |
| `test_create_record_validation_invalid_type` | 不正なレコードタイプのバリデーション | src/web/api.rs:423 |
| `test_create_record_validation_invalid_ttl` | 不正なTTLのバリデーション | src/web/api.rs:450 |
| `test_get_record_not_found` | 存在しないレコードの取得 | src/web/api.rs:477 |
| `test_delete_record` | レコードの削除 | src/web/api.rs:490 |
| `test_get_settings` | 設定一覧の取得 | src/web/api.rs:556 |
| `test_update_setting` | 設定の更新 | src/web/api.rs:582 |
| `test_get_logs_empty` | 空のログ一覧取得 | src/web/api.rs:633 |
| `test_validate_record_ipv6` | IPv6アドレスのバリデーション | src/web/api.rs:654 |
| `test_validate_record_cname` | CNAMEのバリデーション | src/web/api.rs:675 |
| `test_validate_record_empty_content` | 空コンテンツのバリデーション | src/web/api.rs:696 |
| `test_validate_record_ttl_too_high` | 上限超過TTLのバリデーション | src/web/api.rs:707 |

---

## カバレッジ対象外

以下のファイルはテストが困難なため、カバレッジ対象外としています。

| ファイル | 理由 |
|----------|------|
| main.rs | アプリケーションのエントリーポイント |
| web/router.rs | 静的ファイル配信（include_dir使用） |

---

## 手動テスト項目

DESIGN.md記載のシナリオテスト：

```bash
# ローカルレコードの解決
dig @127.0.0.1 app.local.test

# 上位転送の動作確認
dig @127.0.0.1 google.com

# CNAMEレコードの解決
dig @127.0.0.1 alias.local.test
```

---

## フロントエンド（Web UI）

Vitest + React Testing Libraryによる自動テストを実装済み。

| 項目 | 状態 |
|------|------|
| 単体テスト | **実装済み（49テスト）** |
| E2Eテスト | 未実装 |
| 手動テスト | 実施可能 |

### 9. api（APIクライアント）

| テスト名 | 説明 | ファイル |
|----------|------|----------|
| `recordsApi.getAll` | レコード一覧取得 | web-ui/src/api.test.ts |
| `recordsApi.create` | レコード作成 | web-ui/src/api.test.ts |
| `recordsApi.update` | レコード更新 | web-ui/src/api.test.ts |
| `recordsApi.delete` | レコード削除 | web-ui/src/api.test.ts |
| `logsApi.getRecent` | ログ一覧取得 | web-ui/src/api.test.ts |
| `settingsApi.getAll` | 設定一覧取得 | web-ui/src/api.test.ts |
| `settingsApi.update` | 設定更新 | web-ui/src/api.test.ts |
| `healthApi.check` | ヘルスチェック | web-ui/src/api.test.ts |
| 各APIのエラーハンドリング | エラー時の例外スロー確認 | web-ui/src/api.test.ts |

### 10. App（メインアプリ）

| テスト名 | 説明 | ファイル |
|----------|------|----------|
| `タイトルが表示される` | アプリタイトルの表示確認 | web-ui/src/App.test.tsx |
| `ナビゲーションリンクが表示される` | ナビゲーションの描画確認 | web-ui/src/App.test.tsx |
| `テーマ切替ボタンが表示される` | ライト/ダーク/システムボタン確認 | web-ui/src/App.test.tsx |
| `デフォルトでダッシュボードが表示される` | 初期ルート確認 | web-ui/src/App.test.tsx |
| `レコード管理ページに遷移できる` | ルーティング確認 | web-ui/src/App.test.tsx |
| `設定ページに遷移できる` | ルーティング確認 | web-ui/src/App.test.tsx |

### 11. Dashboard（ダッシュボード）

| テスト名 | 説明 | ファイル |
|----------|------|----------|
| `ページタイトルが表示される` | 見出しの表示確認 | web-ui/src/pages/Dashboard.test.tsx |
| `システムステータスセクションが表示される` | ステータス表示確認 | web-ui/src/pages/Dashboard.test.tsx |
| `ヘルスチェック結果が表示される` | API結果の表示確認 | web-ui/src/pages/Dashboard.test.tsx |
| `クエリログセクションが表示される` | ログセクション確認 | web-ui/src/pages/Dashboard.test.tsx |
| `クエリログが読み込まれる` | ログデータ表示確認 | web-ui/src/pages/Dashboard.test.tsx |
| `ログの結果タイプが正しく表示される` | LOCAL/FORWARDED表示確認 | web-ui/src/pages/Dashboard.test.tsx |
| `応答時間が表示される` | 応答時間表示確認 | web-ui/src/pages/Dashboard.test.tsx |
| `テーブルヘッダーが正しく表示される` | テーブル構造確認 | web-ui/src/pages/Dashboard.test.tsx |

### 12. Records（レコード管理）

| テスト名 | 説明 | ファイル |
|----------|------|----------|
| `ページタイトルが表示される` | 見出しの表示確認 | web-ui/src/pages/Records.test.tsx |
| `新規レコード追加ボタンが表示される` | ボタン表示確認 | web-ui/src/pages/Records.test.tsx |
| `レコード一覧が読み込まれる` | データ読み込み確認 | web-ui/src/pages/Records.test.tsx |
| `レコードの詳細が正しく表示される` | 各カラム表示確認 | web-ui/src/pages/Records.test.tsx |
| `新規レコードフォームを表示/非表示できる` | フォーム表示切替確認 | web-ui/src/pages/Records.test.tsx |
| `レコード作成フォームに入力できる` | フォーム入力確認 | web-ui/src/pages/Records.test.tsx |
| `レコードタイプを変更できる` | セレクト変更確認 | web-ui/src/pages/Records.test.tsx |
| `レコードを作成できる` | POST API呼び出し確認 | web-ui/src/pages/Records.test.tsx |
| `編集ボタンでフォームが開く` | 編集モード確認 | web-ui/src/pages/Records.test.tsx |
| `削除ボタンでAPIが呼ばれる` | DELETE API呼び出し確認 | web-ui/src/pages/Records.test.tsx |
| `有効/無効ボタンで状態を切り替えられる` | PUT API呼び出し確認 | web-ui/src/pages/Records.test.tsx |

### 13. Settings（設定）

| テスト名 | 説明 | ファイル |
|----------|------|----------|
| `ページタイトルが表示される` | 見出しの表示確認 | web-ui/src/pages/Settings.test.tsx |
| `設定一覧が読み込まれる` | データ読み込み確認 | web-ui/src/pages/Settings.test.tsx |
| `現在の設定値が表示される` | 設定値表示確認 | web-ui/src/pages/Settings.test.tsx |
| `設定の説明が表示される` | ヘルプテキスト確認 | web-ui/src/pages/Settings.test.tsx |
| `保存ボタンが各設定に表示される` | ボタン数確認 | web-ui/src/pages/Settings.test.tsx |
| `未編集の場合は保存ボタンが無効` | ボタン状態確認 | web-ui/src/pages/Settings.test.tsx |
| `入力値を変更すると保存ボタンが有効になる` | ボタン状態変化確認 | web-ui/src/pages/Settings.test.tsx |
| `設定を保存できる` | PUT API呼び出し確認 | web-ui/src/pages/Settings.test.tsx |

---

## E2Eテスト（未実装）

Playwrightを使用したE2Eテストは将来的に実装予定。

**検討中のシナリオ:**
- レコードCRUDフロー
- 設定変更フロー
- クロスブラウザテスト（Chrome, Firefox, Safari）
