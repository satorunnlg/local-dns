# LocalDNS Pro 開発サーバー起動スクリプト

Write-Host "LocalDNS Pro 開発環境を起動しています..." -ForegroundColor Cyan

# バックエンド (Rust) を起動
$backendJob = Start-Job -ScriptBlock {
    Set-Location $using:PWD
    Write-Host "[Backend] Rustサーバーを起動中..." -ForegroundColor Green
    cargo run
}

# フロントエンド (React) を起動
$frontendJob = Start-Job -ScriptBlock {
    Set-Location $using:PWD\web-ui
    Write-Host "[Frontend] React開発サーバーを起動中..." -ForegroundColor Blue
    npm run dev
}

Write-Host ""
Write-Host "起動完了!" -ForegroundColor Green
Write-Host "  - バックエンドAPI: http://localhost:3000" -ForegroundColor Yellow
Write-Host "  - フロントエンド:   http://localhost:5173" -ForegroundColor Yellow
Write-Host ""
Write-Host "停止するには Ctrl+C を押してください" -ForegroundColor Cyan
Write-Host ""

# ジョブの出力をリアルタイムで表示
try {
    while ($true) {
        # バックエンドの出力
        $backendOutput = Receive-Job -Job $backendJob
        if ($backendOutput) {
            $backendOutput | ForEach-Object {
                Write-Host "[Backend] $_" -ForegroundColor Green
            }
        }

        # フロントエンドの出力
        $frontendOutput = Receive-Job -Job $frontendJob
        if ($frontendOutput) {
            $frontendOutput | ForEach-Object {
                Write-Host "[Frontend] $_" -ForegroundColor Blue
            }
        }

        # ジョブが終了した場合
        if ($backendJob.State -eq 'Completed' -or $backendJob.State -eq 'Failed') {
            Write-Host "[Backend] 終了しました" -ForegroundColor Red
            break
        }
        if ($frontendJob.State -eq 'Completed' -or $frontendJob.State -eq 'Failed') {
            Write-Host "[Frontend] 終了しました" -ForegroundColor Red
            break
        }

        Start-Sleep -Milliseconds 100
    }
}
finally {
    # クリーンアップ
    Write-Host ""
    Write-Host "サーバーを停止しています..." -ForegroundColor Yellow

    Stop-Job -Job $backendJob -ErrorAction SilentlyContinue
    Stop-Job -Job $frontendJob -ErrorAction SilentlyContinue

    Remove-Job -Job $backendJob -Force -ErrorAction SilentlyContinue
    Remove-Job -Job $frontendJob -Force -ErrorAction SilentlyContinue

    Write-Host "停止完了" -ForegroundColor Green
}
