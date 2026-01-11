import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '../test/test-utils'
import Dashboard from './Dashboard'
import { createFetchMock } from '../test/mocks'

describe('Dashboard', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', createFetchMock())
  })

  it('ページタイトルが表示される', () => {
    render(<Dashboard />)
    expect(screen.getByText('ダッシュボード')).toBeInTheDocument()
  })

  it('システムステータスセクションが表示される', () => {
    render(<Dashboard />)
    expect(screen.getByText('システムステータス')).toBeInTheDocument()
    expect(screen.getByText('サービス状態')).toBeInTheDocument()
    expect(screen.getByText('サービス名')).toBeInTheDocument()
  })

  it('ヘルスチェック結果が表示される', async () => {
    render(<Dashboard />)

    expect(await screen.findByText('稼働中')).toBeInTheDocument()
    expect(await screen.findByText('LocalDNS Pro')).toBeInTheDocument()
  })

  it('クエリログセクションが表示される', () => {
    render(<Dashboard />)
    expect(screen.getByText('最近のクエリログ')).toBeInTheDocument()
    expect(screen.getByText('直近100件のDNS問い合わせ履歴')).toBeInTheDocument()
  })

  it('クエリログが読み込まれる', async () => {
    render(<Dashboard />)

    // ログが表示される
    expect(await screen.findByText('test.local')).toBeInTheDocument()
    expect(screen.getByText('google.com')).toBeInTheDocument()
  })

  it('ログの結果タイプが正しく表示される', async () => {
    render(<Dashboard />)

    await screen.findByText('test.local')

    expect(screen.getByText('LOCAL')).toBeInTheDocument()
    expect(screen.getByText('FORWARDED')).toBeInTheDocument()
  })

  it('応答時間が表示される', async () => {
    render(<Dashboard />)

    await screen.findByText('test.local')

    expect(screen.getByText('5ms')).toBeInTheDocument()
    expect(screen.getByText('50ms')).toBeInTheDocument()
  })

  it('テーブルヘッダーが正しく表示される', async () => {
    render(<Dashboard />)

    await screen.findByText('test.local')

    expect(screen.getByText('ドメイン')).toBeInTheDocument()
    expect(screen.getByText('タイプ')).toBeInTheDocument()
    expect(screen.getByText('結果')).toBeInTheDocument()
    expect(screen.getByText('応答時間')).toBeInTheDocument()
    expect(screen.getByText('日時')).toBeInTheDocument()
  })
})
