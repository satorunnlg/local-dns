import { describe, it, expect, vi, beforeEach } from 'vitest'
import { recordsApi, logsApi, settingsApi, healthApi } from './api'
import { mockRecords, mockLogs, mockSettings, mockHealth } from './test/mocks'

describe('API', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  describe('recordsApi', () => {
    it('getAll - レコード一覧を取得できる', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({
          ok: true,
          json: () => Promise.resolve(mockRecords),
        })
      )

      const records = await recordsApi.getAll()
      expect(records).toEqual(mockRecords)
      expect(fetch).toHaveBeenCalledWith('/api/records')
    })

    it('getAll - エラー時に例外をスローする', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({ ok: false, status: 500 })
      )

      await expect(recordsApi.getAll()).rejects.toThrow('レコード取得に失敗しました')
    })

    it('create - レコードを作成できる', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({
          ok: true,
          json: () => Promise.resolve({ id: 1 }),
        })
      )

      const result = await recordsApi.create({
        domain_pattern: 'test.local',
        record_type: 'A',
        content: '127.0.0.1',
        ttl: 60,
      })

      expect(result).toEqual({ id: 1 })
      expect(fetch).toHaveBeenCalledWith('/api/records', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          domain_pattern: 'test.local',
          record_type: 'A',
          content: '127.0.0.1',
          ttl: 60,
        }),
      })
    })

    it('create - エラー時に例外をスローする', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({ ok: false, status: 400 })
      )

      await expect(
        recordsApi.create({
          domain_pattern: '',
          record_type: 'A',
          content: '127.0.0.1',
          ttl: 60,
        })
      ).rejects.toThrow('レコード作成に失敗しました')
    })

    it('update - レコードを更新できる', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({ ok: true })
      )

      await recordsApi.update(1, { content: '10.0.0.1' })

      expect(fetch).toHaveBeenCalledWith('/api/records/1', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content: '10.0.0.1' }),
      })
    })

    it('update - エラー時に例外をスローする', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({ ok: false, status: 404 })
      )

      await expect(recordsApi.update(999, { content: '10.0.0.1' })).rejects.toThrow(
        'レコード更新に失敗しました'
      )
    })

    it('delete - レコードを削除できる', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({ ok: true })
      )

      await recordsApi.delete(1)

      expect(fetch).toHaveBeenCalledWith('/api/records/1', {
        method: 'DELETE',
      })
    })

    it('delete - エラー時に例外をスローする', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({ ok: false, status: 404 })
      )

      await expect(recordsApi.delete(999)).rejects.toThrow('レコード削除に失敗しました')
    })
  })

  describe('logsApi', () => {
    it('getRecent - ログ一覧を取得できる', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({
          ok: true,
          json: () => Promise.resolve(mockLogs),
        })
      )

      const logs = await logsApi.getRecent()
      expect(logs).toEqual(mockLogs)
      expect(fetch).toHaveBeenCalledWith('/api/logs')
    })

    it('getRecent - エラー時に例外をスローする', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({ ok: false, status: 500 })
      )

      await expect(logsApi.getRecent()).rejects.toThrow('ログ取得に失敗しました')
    })
  })

  describe('settingsApi', () => {
    it('getAll - 設定一覧を取得できる', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({
          ok: true,
          json: () => Promise.resolve(mockSettings),
        })
      )

      const settings = await settingsApi.getAll()
      expect(settings).toEqual(mockSettings)
      expect(fetch).toHaveBeenCalledWith('/api/settings')
    })

    it('getAll - エラー時に例外をスローする', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({ ok: false, status: 500 })
      )

      await expect(settingsApi.getAll()).rejects.toThrow('設定取得に失敗しました')
    })

    it('update - 設定を更新できる', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({ ok: true })
      )

      await settingsApi.update('upstream_primary', { value: '1.2.3.4:53' })

      expect(fetch).toHaveBeenCalledWith('/api/settings/upstream_primary', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ value: '1.2.3.4:53' }),
      })
    })

    it('update - エラー時に例外をスローする', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({ ok: false, status: 400 })
      )

      await expect(
        settingsApi.update('unknown_key', { value: 'test' })
      ).rejects.toThrow('設定更新に失敗しました')
    })
  })

  describe('healthApi', () => {
    it('check - ヘルスチェックを実行できる', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({
          ok: true,
          json: () => Promise.resolve(mockHealth),
        })
      )

      const health = await healthApi.check()
      expect(health).toEqual(mockHealth)
      expect(fetch).toHaveBeenCalledWith('/api/health')
    })

    it('check - エラー時に例外をスローする', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({ ok: false, status: 503 })
      )

      await expect(healthApi.check()).rejects.toThrow('ヘルスチェックに失敗しました')
    })
  })
})
