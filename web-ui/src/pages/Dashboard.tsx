import { useQuery } from '@tanstack/react-query'
import { logsApi, healthApi } from '../api'

function Dashboard() {
  const { data: health } = useQuery({
    queryKey: ['health'],
    queryFn: healthApi.check,
    refetchInterval: 5000,
  })

  const { data: logs, isLoading } = useQuery({
    queryKey: ['logs'],
    queryFn: logsApi.getRecent,
    refetchInterval: 3000,
  })

  return (
    <div className="px-4 py-6 sm:px-0">
      <h2 className="text-2xl font-bold mb-6 text-gray-900 dark:text-white">ダッシュボード</h2>

      {/* ステータス */}
      <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg mb-6 transition-colors">
        <div className="px-4 py-5 sm:p-6">
          <h3 className="text-lg font-medium leading-6 text-gray-900 dark:text-white mb-4">
            システムステータス
          </h3>
          <div className="grid grid-cols-1 gap-5 sm:grid-cols-2">
            <div className="bg-green-50 dark:bg-green-900/30 overflow-hidden shadow rounded-lg">
              <div className="px-4 py-5 sm:p-6">
                <dt className="text-sm font-medium text-green-900 dark:text-green-300 truncate">
                  サービス状態
                </dt>
                <dd className="mt-1 text-3xl font-semibold text-green-600 dark:text-green-400">
                  {health?.status === 'ok' ? '稼働中' : '不明'}
                </dd>
              </div>
            </div>
            <div className="bg-blue-50 dark:bg-blue-900/30 overflow-hidden shadow rounded-lg">
              <div className="px-4 py-5 sm:p-6">
                <dt className="text-sm font-medium text-blue-900 dark:text-blue-300 truncate">
                  サービス名
                </dt>
                <dd className="mt-1 text-3xl font-semibold text-blue-600 dark:text-blue-400">
                  {health?.service || '-'}
                </dd>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* クエリログ */}
      <div className="bg-white dark:bg-gray-800 shadow overflow-hidden sm:rounded-lg transition-colors">
        <div className="px-4 py-5 sm:px-6">
          <h3 className="text-lg leading-6 font-medium text-gray-900 dark:text-white">
            最近のクエリログ
          </h3>
          <p className="mt-1 max-w-2xl text-sm text-gray-500 dark:text-gray-400">
            直近100件のDNS問い合わせ履歴
          </p>
        </div>
        <div className="border-t border-gray-200 dark:border-gray-700">
          {isLoading ? (
            <div className="px-4 py-5 sm:px-6 text-center text-gray-500 dark:text-gray-400">
              読み込み中...
            </div>
          ) : logs && logs.length > 0 ? (
            <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead className="bg-gray-50 dark:bg-gray-700">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    ドメイン
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    タイプ
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    結果
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    応答時間
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    日時
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                {logs.map((log) => (
                  <tr key={log.id}>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">
                      {log.query_name}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                      <span className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-blue-100 dark:bg-blue-900/50 text-blue-800 dark:text-blue-300">
                        {log.q_type}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                      <span
                        className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${
                          log.result_type === 'LOCAL'
                            ? 'bg-green-100 dark:bg-green-900/50 text-green-800 dark:text-green-300'
                            : log.result_type === 'FORWARDED'
                            ? 'bg-yellow-100 dark:bg-yellow-900/50 text-yellow-800 dark:text-yellow-300'
                            : 'bg-red-100 dark:bg-red-900/50 text-red-800 dark:text-red-300'
                        }`}
                      >
                        {log.result_type}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                      {log.duration_ms}ms
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                      {new Date(log.timestamp).toLocaleString('ja-JP')}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : (
            <div className="px-4 py-5 sm:px-6 text-center text-gray-500 dark:text-gray-400">
              ログがありません
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default Dashboard
