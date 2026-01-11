import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '../test/test-utils'
import userEvent from '@testing-library/user-event'
import Settings from './Settings'
import { createFetchMock } from '../test/mocks'

describe('Settings', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', createFetchMock())
  })

  it('ページタイトルが表示される', () => {
    render(<Settings />)
    expect(screen.getByText('設定')).toBeInTheDocument()
    expect(screen.getByText('システム設定の変更が行えます')).toBeInTheDocument()
  })

  it('設定一覧が読み込まれる', async () => {
    render(<Settings />)

    // 読み込み中表示
    expect(screen.getByText('読み込み中...')).toBeInTheDocument()

    // 設定が表示される
    expect(await screen.findByText('プライマリDNS')).toBeInTheDocument()
    expect(screen.getByText('セカンダリDNS')).toBeInTheDocument()
    expect(screen.getByText('タイムアウト (ミリ秒)')).toBeInTheDocument()
    expect(screen.getByText('ログ保存期間 (日)')).toBeInTheDocument()
  })

  it('現在の設定値が表示される', async () => {
    render(<Settings />)

    await screen.findByText('プライマリDNS')

    expect(screen.getByText('現在の値: 8.8.8.8:53')).toBeInTheDocument()
    expect(screen.getByText('現在の値: 1.1.1.1:53')).toBeInTheDocument()
    expect(screen.getByText('現在の値: 2000')).toBeInTheDocument()
    expect(screen.getByText('現在の値: 7')).toBeInTheDocument()
  })

  it('設定の説明が表示される', async () => {
    render(<Settings />)

    await screen.findByText('プライマリDNS')

    expect(screen.getByText(/最初に問い合わせるDNSサーバー/)).toBeInTheDocument()
    expect(screen.getByText(/プライマリが失敗した場合に使用する/)).toBeInTheDocument()
    expect(screen.getByText(/上位DNSへの問い合わせタイムアウト時間/)).toBeInTheDocument()
    expect(screen.getByText(/この日数を超えた古いログは自動削除されます/)).toBeInTheDocument()
  })

  it('保存ボタンが各設定に表示される', async () => {
    render(<Settings />)

    await screen.findByText('プライマリDNS')

    const saveButtons = screen.getAllByRole('button', { name: '保存' })
    expect(saveButtons).toHaveLength(4)
  })

  it('未編集の場合は保存ボタンが無効', async () => {
    render(<Settings />)

    await screen.findByText('プライマリDNS')

    const saveButtons = screen.getAllByRole('button', { name: '保存' })
    saveButtons.forEach((button) => {
      expect(button).toBeDisabled()
    })
  })

  it('入力値を変更すると保存ボタンが有効になる', async () => {
    const user = userEvent.setup()
    render(<Settings />)

    await screen.findByText('プライマリDNS')

    const inputs = screen.getAllByRole('textbox')
    await user.clear(inputs[0])
    await user.type(inputs[0], '1.2.3.4:53')

    const saveButtons = screen.getAllByRole('button', { name: '保存' })
    expect(saveButtons[0]).not.toBeDisabled()
  })

  it('設定を保存できる', async () => {
    const user = userEvent.setup()
    const fetchMock = createFetchMock()
    vi.stubGlobal('fetch', fetchMock)

    render(<Settings />)

    await screen.findByText('プライマリDNS')

    const inputs = screen.getAllByRole('textbox')
    await user.clear(inputs[0])
    await user.type(inputs[0], '1.2.3.4:53')

    const saveButtons = screen.getAllByRole('button', { name: '保存' })
    await user.click(saveButtons[0])

    await waitFor(() => {
      const putCall = fetchMock.mock.calls.find(
        (call) =>
          call[0].includes('/api/settings/upstream_primary') &&
          call[1]?.method === 'PUT'
      )
      expect(putCall).toBeDefined()
    })
  })
})
