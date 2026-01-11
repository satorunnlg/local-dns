import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from './test/test-utils'
import userEvent from '@testing-library/user-event'
import App from './App'
import { createFetchMock } from './test/mocks'

describe('App', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', createFetchMock())
  })

  // Appコンポーネントは独自のBrowserRouterを持つためskipRouterを指定
  const renderApp = () => render(<App />, { skipRouter: true })

  it('タイトルが表示される', () => {
    renderApp()
    expect(screen.getByText('LocalDNS Pro')).toBeInTheDocument()
  })

  it('ナビゲーションリンクが表示される', () => {
    renderApp()
    // ナビゲーションリンクを取得（roleがlinkでテキストで検索）
    expect(screen.getByRole('link', { name: 'ダッシュボード' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'レコード管理' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: '設定' })).toBeInTheDocument()
  })

  it('テーマ切替ボタンが表示される', () => {
    renderApp()
    // テーマ切替ボタンはtitle属性で検出
    expect(screen.getByRole('button', { name: 'ライト' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'ダーク' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'システム' })).toBeInTheDocument()
  })

  it('デフォルトでダッシュボードが表示される', async () => {
    renderApp()
    expect(await screen.findByText('ダッシュボード', { selector: 'h2' })).toBeInTheDocument()
  })

  it('レコード管理ページに遷移できる', async () => {
    const user = userEvent.setup()
    renderApp()

    await user.click(screen.getByText('レコード管理'))
    expect(await screen.findByText('レコード管理', { selector: 'h2' })).toBeInTheDocument()
  })

  it('設定ページに遷移できる', async () => {
    const user = userEvent.setup()
    renderApp()

    await user.click(screen.getByText('設定'))
    expect(await screen.findByText('設定', { selector: 'h2' })).toBeInTheDocument()
  })
})
