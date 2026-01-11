import { ReactElement } from 'react'
import { render, RenderOptions } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider } from '../components/ThemeProvider'

// テスト用のQueryClient（キャッシュやリトライを無効化）
function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
        staleTime: 0,
      },
      mutations: {
        retry: false,
      },
    },
  })
}

interface WrapperProps {
  children: React.ReactNode
}

interface CustomRenderOptions extends Omit<RenderOptions, 'wrapper'> {
  // BrowserRouterをスキップするオプション（Appコンポーネントなど独自のRouterを持つ場合）
  skipRouter?: boolean
}

// カスタムレンダー関数
function customRender(
  ui: ReactElement,
  options?: CustomRenderOptions
) {
  const { skipRouter = false, ...renderOptions } = options ?? {}
  const queryClient = createTestQueryClient()

  function Wrapper({ children }: WrapperProps) {
    const content = skipRouter ? children : <BrowserRouter>{children}</BrowserRouter>
    return (
      <QueryClientProvider client={queryClient}>
        <ThemeProvider>
          {content}
        </ThemeProvider>
      </QueryClientProvider>
    )
  }

  return {
    ...render(ui, { wrapper: Wrapper, ...renderOptions }),
    queryClient,
  }
}

// すべてのexportを再エクスポート
export * from '@testing-library/react'
export { customRender as render }
