import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '../test/test-utils'
import userEvent from '@testing-library/user-event'
import Records from './Records'
import { createFetchMock, mockRecords } from '../test/mocks'

describe('Records', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', createFetchMock())
  })

  it('ページタイトルが表示される', () => {
    render(<Records />)
    expect(screen.getByText('レコード管理')).toBeInTheDocument()
  })

  it('新規レコード追加ボタンが表示される', () => {
    render(<Records />)
    expect(screen.getByRole('button', { name: '新規レコード追加' })).toBeInTheDocument()
  })

  it('レコード一覧が読み込まれる', async () => {
    render(<Records />)

    // 読み込み中表示
    expect(screen.getByText('読み込み中...')).toBeInTheDocument()

    // レコードが表示される
    expect(await screen.findByText('test.local')).toBeInTheDocument()
    expect(screen.getByText('%.example.com')).toBeInTheDocument()
  })

  it('レコードの詳細が正しく表示される', async () => {
    render(<Records />)

    await screen.findByText('test.local')

    // 各カラムの内容を確認
    expect(screen.getByText('127.0.0.1')).toBeInTheDocument()
    expect(screen.getByText('192.168.1.1')).toBeInTheDocument()
    expect(screen.getAllByText('A')).toHaveLength(2)
  })

  it('新規レコードフォームを表示/非表示できる', async () => {
    const user = userEvent.setup()
    render(<Records />)

    // フォームは最初非表示
    expect(screen.queryByPlaceholderText(/app\.local\.test/)).not.toBeInTheDocument()

    // ボタンクリックでフォーム表示
    await user.click(screen.getByRole('button', { name: '新規レコード追加' }))
    expect(screen.getByPlaceholderText(/app\.local\.test/)).toBeInTheDocument()

    // フォーム内のキャンセルボタンで非表示（getAllByRoleで複数取得し、最初のものを使用）
    const cancelButtons = screen.getAllByRole('button', { name: 'キャンセル' })
    await user.click(cancelButtons[0])
    expect(screen.queryByPlaceholderText(/app\.local\.test/)).not.toBeInTheDocument()
  })

  it('レコード作成フォームに入力できる', async () => {
    const user = userEvent.setup()
    render(<Records />)

    await user.click(screen.getByRole('button', { name: '新規レコード追加' }))

    // ドメインパターン入力
    const domainInput = screen.getByPlaceholderText(/app\.local\.test/)
    await user.type(domainInput, 'new.local.test')
    expect(domainInput).toHaveValue('new.local.test')

    // コンテンツ入力
    const contentInput = screen.getByPlaceholderText(/127\.0\.0\.1/)
    await user.type(contentInput, '10.0.0.1')
    expect(contentInput).toHaveValue('10.0.0.1')
  })

  it('レコードタイプを変更できる', async () => {
    const user = userEvent.setup()
    render(<Records />)

    await user.click(screen.getByRole('button', { name: '新規レコード追加' }))

    const select = screen.getByRole('combobox')
    await user.selectOptions(select, 'AAAA')
    expect(select).toHaveValue('AAAA')

    await user.selectOptions(select, 'CNAME')
    expect(select).toHaveValue('CNAME')
  })

  it('レコードを作成できる', async () => {
    const user = userEvent.setup()
    const fetchMock = createFetchMock()
    vi.stubGlobal('fetch', fetchMock)

    render(<Records />)

    await user.click(screen.getByRole('button', { name: '新規レコード追加' }))

    await user.type(screen.getByPlaceholderText(/app\.local\.test/), 'new.local.test')
    await user.type(screen.getByPlaceholderText(/127\.0\.0\.1/), '10.0.0.1')

    await user.click(screen.getByRole('button', { name: '作成' }))

    // APIが呼ばれたことを確認
    await waitFor(() => {
      const postCall = fetchMock.mock.calls.find(
        (call) => call[0].includes('/api/records') && call[1]?.method === 'POST'
      )
      expect(postCall).toBeDefined()
    })
  })

  it('編集ボタンでフォームが開く', async () => {
    const user = userEvent.setup()
    render(<Records />)

    await screen.findByText('test.local')

    const editButtons = screen.getAllByRole('button', { name: '編集' })
    await user.click(editButtons[0])

    // フォームが編集モードで表示される
    expect(screen.getByText('レコード編集')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '更新' })).toBeInTheDocument()
  })

  it('削除ボタンでAPIが呼ばれる', async () => {
    const user = userEvent.setup()
    const fetchMock = createFetchMock()
    vi.stubGlobal('fetch', fetchMock)

    render(<Records />)

    await screen.findByText('test.local')

    const deleteButtons = screen.getAllByRole('button', { name: '削除' })
    await user.click(deleteButtons[0])

    await waitFor(() => {
      const deleteCall = fetchMock.mock.calls.find(
        (call) => call[0].includes('/api/records/1') && call[1]?.method === 'DELETE'
      )
      expect(deleteCall).toBeDefined()
    })
  })

  it('有効/無効ボタンで状態を切り替えられる', async () => {
    const user = userEvent.setup()
    const fetchMock = createFetchMock()
    vi.stubGlobal('fetch', fetchMock)

    render(<Records />)

    await screen.findByText('test.local')

    const toggleButtons = screen.getAllByRole('button', { name: '有効' })
    await user.click(toggleButtons[0])

    await waitFor(() => {
      const putCall = fetchMock.mock.calls.find(
        (call) => call[0].includes('/api/records/1') && call[1]?.method === 'PUT'
      )
      expect(putCall).toBeDefined()
    })
  })
})
