import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { recordsApi } from '../api'
import type { CreateRecordRequest, Record } from '../types'

function Records() {
  const queryClient = useQueryClient()
  const [showForm, setShowForm] = useState(false)
  const [editingId, setEditingId] = useState<number | null>(null)
  const [formData, setFormData] = useState<CreateRecordRequest>({
    domain_pattern: '',
    record_type: 'A',
    content: '',
    ttl: 60,
  })

  const { data: records, isLoading } = useQuery({
    queryKey: ['records'],
    queryFn: recordsApi.getAll,
  })

  const createMutation = useMutation({
    mutationFn: recordsApi.create,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['records'] })
      setShowForm(false)
      setEditingId(null)
      setFormData({ domain_pattern: '', record_type: 'A', content: '', ttl: 60 })
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: number; data: Partial<Record> }) =>
      recordsApi.update(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['records'] })
      setShowForm(false)
      setEditingId(null)
      setFormData({ domain_pattern: '', record_type: 'A', content: '', ttl: 60 })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: recordsApi.delete,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['records'] })
    },
  })

  const toggleMutation = useMutation({
    mutationFn: ({ id, active }: { id: number; active: number }) =>
      recordsApi.update(id, { active: active === 1 ? 0 : 1 }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['records'] })
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (editingId !== null) {
      updateMutation.mutate({ id: editingId, data: formData })
    } else {
      createMutation.mutate(formData)
    }
  }

  const handleEdit = (record: Record) => {
    setEditingId(record.id)
    setFormData({
      domain_pattern: record.domain_pattern,
      record_type: record.record_type,
      content: record.content,
      ttl: record.ttl,
    })
    setShowForm(true)
  }

  const handleCancel = () => {
    setShowForm(false)
    setEditingId(null)
    setFormData({ domain_pattern: '', record_type: 'A', content: '', ttl: 60 })
  }

  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h2 className="text-2xl font-bold">レコード管理</h2>
          <p className="mt-2 text-sm text-gray-700">
            DNSレコードの追加、編集、削除が行えます
          </p>
        </div>
        <div className="mt-4 sm:mt-0 sm:ml-16 sm:flex-none">
          <button
            type="button"
            onClick={() => (showForm ? handleCancel() : setShowForm(true))}
            className="inline-flex items-center justify-center rounded-md border border-transparent bg-blue-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 sm:w-auto"
          >
            {showForm ? 'キャンセル' : '新規レコード追加'}
          </button>
        </div>
      </div>

      {showForm && (
        <div className="mt-6 bg-white shadow sm:rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <h3 className="text-lg font-medium leading-6 text-gray-900">
              {editingId !== null ? 'レコード編集' : '新規レコード'}
            </h3>
            <form onSubmit={handleSubmit} className="mt-5 space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700">
                  ドメインパターン
                </label>
                <input
                  type="text"
                  required
                  value={formData.domain_pattern}
                  onChange={(e) =>
                    setFormData({ ...formData, domain_pattern: e.target.value })
                  }
                  placeholder="app.local.test または %.local.test"
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm px-3 py-2 border"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">
                  レコードタイプ
                </label>
                <select
                  value={formData.record_type}
                  onChange={(e) =>
                    setFormData({ ...formData, record_type: e.target.value })
                  }
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm px-3 py-2 border"
                >
                  <option value="A">A (IPv4)</option>
                  <option value="AAAA">AAAA (IPv6)</option>
                  <option value="CNAME">CNAME</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">
                  コンテンツ
                </label>
                <input
                  type="text"
                  required
                  value={formData.content}
                  onChange={(e) =>
                    setFormData({ ...formData, content: e.target.value })
                  }
                  placeholder="127.0.0.1 または target.example.com"
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm px-3 py-2 border"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">
                  TTL (秒)
                </label>
                <input
                  type="number"
                  required
                  value={formData.ttl}
                  onChange={(e) =>
                    setFormData({ ...formData, ttl: parseInt(e.target.value) })
                  }
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm px-3 py-2 border"
                />
              </div>
              <div className="flex gap-2">
                <button
                  type="submit"
                  disabled={createMutation.isPending || updateMutation.isPending}
                  className="inline-flex justify-center rounded-md border border-transparent bg-blue-600 py-2 px-4 text-sm font-medium text-white shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                >
                  {editingId !== null
                    ? updateMutation.isPending
                      ? '更新中...'
                      : '更新'
                    : createMutation.isPending
                    ? '作成中...'
                    : '作成'}
                </button>
                <button
                  type="button"
                  onClick={handleCancel}
                  className="inline-flex justify-center rounded-md border border-gray-300 bg-white py-2 px-4 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                >
                  キャンセル
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      <div className="mt-8 flex flex-col">
        <div className="-my-2 -mx-4 overflow-x-auto sm:-mx-6 lg:-mx-8">
          <div className="inline-block min-w-full py-2 align-middle md:px-6 lg:px-8">
            <div className="overflow-hidden shadow ring-1 ring-black ring-opacity-5 md:rounded-lg">
              {isLoading ? (
                <div className="bg-white px-4 py-5 sm:px-6 text-center text-gray-500">
                  読み込み中...
                </div>
              ) : records && records.length > 0 ? (
                <table className="min-w-full divide-y divide-gray-300">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900 sm:pl-6">
                        ドメインパターン
                      </th>
                      <th className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                        タイプ
                      </th>
                      <th className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                        コンテンツ
                      </th>
                      <th className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                        TTL
                      </th>
                      <th className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                        状態
                      </th>
                      <th className="relative py-3.5 pl-3 pr-4 sm:pr-6">
                        <span className="sr-only">操作</span>
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-gray-200 bg-white">
                    {records.map((record) => (
                      <tr key={record.id}>
                        <td className="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-gray-900 sm:pl-6">
                          {record.domain_pattern}
                        </td>
                        <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                          <span className="inline-flex rounded-full bg-blue-100 px-2 text-xs font-semibold leading-5 text-blue-800">
                            {record.record_type}
                          </span>
                        </td>
                        <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                          {record.content}
                        </td>
                        <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                          {record.ttl}
                        </td>
                        <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                          <button
                            onClick={() =>
                              toggleMutation.mutate({
                                id: record.id,
                                active: record.active,
                              })
                            }
                            className={`inline-flex rounded-full px-2 text-xs font-semibold leading-5 ${
                              record.active === 1
                                ? 'bg-green-100 text-green-800'
                                : 'bg-gray-100 text-gray-800'
                            }`}
                          >
                            {record.active === 1 ? '有効' : '無効'}
                          </button>
                        </td>
                        <td className="relative whitespace-nowrap py-4 pl-3 pr-4 text-right text-sm font-medium sm:pr-6">
                          <button
                            onClick={() => handleEdit(record)}
                            className="text-blue-600 hover:text-blue-900 mr-4"
                          >
                            編集
                          </button>
                          <button
                            onClick={() => deleteMutation.mutate(record.id)}
                            className="text-red-600 hover:text-red-900"
                          >
                            削除
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              ) : (
                <div className="bg-white px-4 py-5 sm:px-6 text-center text-gray-500">
                  レコードがありません
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Records
