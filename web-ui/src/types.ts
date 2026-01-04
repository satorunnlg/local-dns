// レコード型
export interface Record {
  id: number
  domain_pattern: string
  record_type: string
  content: string
  ttl: number
  active: number
}

// クエリログ型
export interface QueryLog {
  id: number
  query_name: string
  q_type: string
  result_type: string
  duration_ms: number
  timestamp: string
}

// 設定型
export interface Setting {
  key: string
  value: string
}

// レコード作成リクエスト
export interface CreateRecordRequest {
  domain_pattern: string
  record_type: string
  content: string
  ttl?: number
}

// レコード更新リクエスト
export interface UpdateRecordRequest {
  domain_pattern?: string
  record_type?: string
  content?: string
  ttl?: number
  active?: number
}

// 設定更新リクエスト
export interface UpdateSettingRequest {
  value: string
}
