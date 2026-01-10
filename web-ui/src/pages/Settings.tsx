import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { settingsApi } from '../api'

function Settings() {
  const queryClient = useQueryClient()
  const [editValues, setEditValues] = useState<Record<string, string>>({})

  const { data: settings, isLoading } = useQuery({
    queryKey: ['settings'],
    queryFn: settingsApi.getAll,
  })

  const updateMutation = useMutation({
    mutationFn: ({ key, value }: { key: string; value: string }) =>
      settingsApi.update(key, { value }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings'] })
      setEditValues({})
    },
  })

  const handleUpdate = (key: string) => {
    const value = editValues[key]
    if (value !== undefined) {
      updateMutation.mutate({ key, value })
    }
  }

  const getSettingLabel = (key: string): string => {
    switch (key) {
      case 'upstream_primary':
        return 'プライマリDNS'
      case 'upstream_secondary':
        return 'セカンダリDNS'
      case 'upstream_timeout_ms':
        return 'タイムアウト (ミリ秒)'
      case 'log_retention_days':
        return 'ログ保存期間 (日)'
      default:
        return key
    }
  }

  const getSettingDescription = (key: string): string => {
    switch (key) {
      case 'upstream_primary':
        return '最初に問い合わせるDNSサーバー (例: 8.8.8.8:53)'
      case 'upstream_secondary':
        return 'プライマリが失敗した場合に使用するDNSサーバー (例: 1.1.1.1:53)'
      case 'upstream_timeout_ms':
        return '上位DNSへの問い合わせタイムアウト時間'
      case 'log_retention_days':
        return 'この日数を超えた古いログは自動削除されます'
      default:
        return ''
    }
  }

  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">設定</h2>
          <p className="mt-2 text-sm text-gray-700 dark:text-gray-300">
            システム設定の変更が行えます
          </p>
        </div>
      </div>

      <div className="mt-8">
        {isLoading ? (
          <div className="bg-white dark:bg-gray-800 px-4 py-5 sm:px-6 text-center text-gray-500 dark:text-gray-400 rounded-lg shadow transition-colors">
            読み込み中...
          </div>
        ) : settings && settings.length > 0 ? (
          <div className="space-y-6">
            {settings.map((setting) => (
              <div
                key={setting.key}
                className="bg-white dark:bg-gray-800 shadow sm:rounded-lg transition-colors"
              >
                <div className="px-4 py-5 sm:p-6">
                  <h3 className="text-lg font-medium leading-6 text-gray-900 dark:text-white">
                    {getSettingLabel(setting.key)}
                  </h3>
                  <div className="mt-2 max-w-xl text-sm text-gray-500 dark:text-gray-400">
                    <p>{getSettingDescription(setting.key)}</p>
                  </div>
                  <div className="mt-5 sm:flex sm:items-center">
                    <div className="w-full sm:max-w-xs">
                      <input
                        type="text"
                        value={
                          editValues[setting.key] !== undefined
                            ? editValues[setting.key]
                            : setting.value
                        }
                        onChange={(e) =>
                          setEditValues({
                            ...editValues,
                            [setting.key]: e.target.value,
                          })
                        }
                        className="block w-full rounded-md border-gray-300 dark:border-gray-600 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm px-3 py-2 border bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                      />
                    </div>
                    <button
                      type="button"
                      onClick={() => handleUpdate(setting.key)}
                      disabled={
                        updateMutation.isPending ||
                        editValues[setting.key] === undefined
                      }
                      className="mt-3 inline-flex w-full items-center justify-center rounded-md border border-transparent bg-blue-600 px-4 py-2 font-medium text-white shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm disabled:opacity-50"
                    >
                      {updateMutation.isPending ? '保存中...' : '保存'}
                    </button>
                  </div>
                  <div className="mt-2 text-sm text-gray-500 dark:text-gray-400">
                    現在の値: {setting.value}
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="bg-white dark:bg-gray-800 px-4 py-5 sm:px-6 text-center text-gray-500 dark:text-gray-400 rounded-lg shadow transition-colors">
            設定がありません
          </div>
        )}
      </div>
    </div>
  )
}

export default Settings
