import type {
  Record,
  QueryLog,
  Setting,
  CreateRecordRequest,
  UpdateRecordRequest,
  UpdateSettingRequest,
} from './types'

const API_BASE = '/api'

// レコード関連API
export const recordsApi = {
  getAll: async (): Promise<Record[]> => {
    const res = await fetch(`${API_BASE}/records`)
    if (!res.ok) throw new Error('レコード取得に失敗しました')
    return res.json()
  },

  create: async (data: CreateRecordRequest): Promise<{ id: number }> => {
    const res = await fetch(`${API_BASE}/records`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    })
    if (!res.ok) throw new Error('レコード作成に失敗しました')
    return res.json()
  },

  update: async (id: number, data: UpdateRecordRequest): Promise<void> => {
    const res = await fetch(`${API_BASE}/records/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    })
    if (!res.ok) throw new Error('レコード更新に失敗しました')
  },

  delete: async (id: number): Promise<void> => {
    const res = await fetch(`${API_BASE}/records/${id}`, {
      method: 'DELETE',
    })
    if (!res.ok) throw new Error('レコード削除に失敗しました')
  },
}

// ログ関連API
export const logsApi = {
  getRecent: async (): Promise<QueryLog[]> => {
    const res = await fetch(`${API_BASE}/logs`)
    if (!res.ok) throw new Error('ログ取得に失敗しました')
    return res.json()
  },
}

// 設定関連API
export const settingsApi = {
  getAll: async (): Promise<Setting[]> => {
    const res = await fetch(`${API_BASE}/settings`)
    if (!res.ok) throw new Error('設定取得に失敗しました')
    return res.json()
  },

  update: async (key: string, data: UpdateSettingRequest): Promise<void> => {
    const res = await fetch(`${API_BASE}/settings/${key}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    })
    if (!res.ok) throw new Error('設定更新に失敗しました')
  },
}

// ヘルスチェック
export const healthApi = {
  check: async (): Promise<{ status: string; service: string }> => {
    const res = await fetch(`${API_BASE}/health`)
    if (!res.ok) throw new Error('ヘルスチェックに失敗しました')
    return res.json()
  },
}
