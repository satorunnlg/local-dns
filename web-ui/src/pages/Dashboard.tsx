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
      <h2 className="text-2xl font-bold mb-6">ダッシュボード</h2>

      {/* ステータス */}
      <div className="bg-white overflow-hidden shadow rounded-lg mb-6">
        <div className="px-4 py-5 sm:p-6">
          <h3 className="text-lg font-medium leading-6 text-gray-900 mb-4">
            システムステータス
          </h3>
          <div className="grid grid-cols-1 gap-5 sm:grid-cols-2">
            <div className="bg-green-50 overflow-hidden shadow rounded-lg">
              <div className="px-4 py-5 sm:p-6">
                <dt className="text-sm font-medium text-green-900 truncate">
                  サービス状態
                </dt>
                <dd className="mt-1 text-3xl font-semibold text-green-600">
                  {health?.status === 'ok' ? '稼働中' : '不明'}
                </dd>
              </div>
            </div>
            <div className="bg-blue-50 overflow-hidden shadow rounded-lg">
              <div className="px-4 py-5 sm:p-6">
                <dt className="text-sm font-medium text-blue-900 truncate">
                  サービス名
                </dt>
                <dd className="mt-1 text-3xl font-semibold text-blue-600">
                  {health?.service || '-'}
                </dd>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* クエリログ */}
      <div className="bg-white shadow overflow-hidden sm:rounded-lg">
        <div className="px-4 py-5 sm:px-6">
          <h3 className="text-lg leading-6 font-medium text-gray-900">
            最近のクエリログ
          </h3>
          <p className="mt-1 max-w-2xl text-sm text-gray-500">
            直近100件のDNS問い合わせ履歴
          </p>
        </div>
        <div className="border-t border-gray-200">
          {isLoading ? (
            <div className="px-4 py-5 sm:px-6 text-center text-gray-500">
              読み込み中...
            </div>
          ) : logs && logs.length > 0 ? (
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    ドメイン
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    タイプ
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    結果
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    応答時間
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    日時
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {logs.map((log) => (
                  <tr key={log.id}>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                      {log.query_name}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      <span className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-blue-100 text-blue-800">
                        {log.q_type}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      <span
                        className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${
                          log.result_type === 'LOCAL'
                            ? 'bg-green-100 text-green-800'
                            : log.result_type === 'FORWARDED'
                            ? 'bg-yellow-100 text-yellow-800'
                            : 'bg-red-100 text-red-800'
                        }`}
                      >
                        {log.result_type}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {log.duration_ms}ms
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {new Date(log.timestamp).toLocaleString('ja-JP')}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : (
            <div className="px-4 py-5 sm:px-6 text-center text-gray-500">
              ログがありません
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default Dashboard
