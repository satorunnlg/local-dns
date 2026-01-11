import { vi } from 'vitest'
import type { Record, QueryLog, Setting } from '../types'

// モックデータ
export const mockRecords: Record[] = [
  {
    id: 1,
    domain_pattern: 'test.local',
    record_type: 'A',
    content: '127.0.0.1',
    ttl: 60,
    active: 1,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
  },
  {
    id: 2,
    domain_pattern: '%.example.com',
    record_type: 'A',
    content: '192.168.1.1',
    ttl: 300,
    active: 1,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
  },
]

export const mockLogs: QueryLog[] = [
  {
    id: 1,
    query_name: 'test.local',
    q_type: 'A',
    result_type: 'LOCAL',
    duration_ms: 5,
    timestamp: '2026-01-11T10:00:00Z',
  },
  {
    id: 2,
    query_name: 'google.com',
    q_type: 'A',
    result_type: 'FORWARDED',
    duration_ms: 50,
    timestamp: '2026-01-11T10:01:00Z',
  },
]

export const mockSettings: Setting[] = [
  { key: 'upstream_primary', value: '8.8.8.8:53' },
  { key: 'upstream_secondary', value: '1.1.1.1:53' },
  { key: 'upstream_timeout_ms', value: '2000' },
  { key: 'log_retention_days', value: '7' },
]

export const mockHealth = {
  status: 'ok',
  service: 'LocalDNS Pro',
}

// APIモック作成ヘルパー
export function createFetchMock() {
  return vi.fn((url: string, options?: RequestInit) => {
    const method = options?.method || 'GET'

    // Health API
    if (url.endsWith('/api/health')) {
      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve(mockHealth),
      })
    }

    // Records API
    if (url.endsWith('/api/records') && method === 'GET') {
      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve(mockRecords),
      })
    }

    if (url.endsWith('/api/records') && method === 'POST') {
      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve({ id: 3 }),
      })
    }

    if (url.match(/\/api\/records\/\d+$/) && method === 'PUT') {
      return Promise.resolve({ ok: true })
    }

    if (url.match(/\/api\/records\/\d+$/) && method === 'DELETE') {
      return Promise.resolve({ ok: true })
    }

    // Logs API
    if (url.endsWith('/api/logs')) {
      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve(mockLogs),
      })
    }

    // Settings API
    if (url.endsWith('/api/settings') && method === 'GET') {
      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve(mockSettings),
      })
    }

    if (url.match(/\/api\/settings\//) && method === 'PUT') {
      return Promise.resolve({ ok: true })
    }

    // 404
    return Promise.resolve({
      ok: false,
      status: 404,
    })
  })
}
