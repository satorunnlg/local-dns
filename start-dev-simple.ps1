# LocalDNS Pro 開発サーバー起動スクリプト (シンプル版)
# 各サーバーを別ウィンドウで起動します

Write-Host "LocalDNS Pro 開発環境を起動しています..." -ForegroundColor Cyan
Write-Host ""

# バックエンド (Rust) を別ウィンドウで起動
Write-Host "バックエンドサーバーを起動中..." -ForegroundColor Green
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd '$PWD'; Write-Host 'Backend (Rust) - http://localhost:3000' -ForegroundColor Green; cargo run"

# フロントエンド (React) を別ウィンドウで起動
Write-Host "フロントエンドサーバーを起動中..." -ForegroundColor Blue
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd '$PWD\web-ui'; Write-Host 'Frontend (React) - http://localhost:5173' -ForegroundColor Blue; npm run dev"

Write-Host ""
Write-Host "起動完了!" -ForegroundColor Green
Write-Host "  - バックエンドAPI: http://localhost:3000" -ForegroundColor Yellow
Write-Host "  - フロントエンド:   http://localhost:5173" -ForegroundColor Yellow
Write-Host ""
Write-Host "各ウィンドウで Ctrl+C を押すと停止します" -ForegroundColor Cyan
