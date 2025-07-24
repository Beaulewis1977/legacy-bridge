import '@testing-library/jest-dom'
import { expect, afterEach } from 'vitest'
import { cleanup } from '@testing-library/react'

// Cleanup after each test
afterEach(() => {
  cleanup()
})

// Mock Tauri API calls since they won't work in test environment
const mockTauriApi = {
  convertRtfToMd: vi.fn(),
  processWithPipeline: vi.fn(),
  validateDocument: vi.fn(),
  applyTemplate: vi.fn(),
  getVersionInfo: vi.fn(),
  testConnection: vi.fn(),
  getLastError: vi.fn()
}

// Mock the tauri-api module
vi.mock('@/lib/tauri-api', () => ({
  ...mockTauriApi,
  __esModule: true,
  default: mockTauriApi
}))

// Mock next/navigation
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: vi.fn(),
    back: vi.fn(),
    forward: vi.fn(),
    refresh: vi.fn(),
    replace: vi.fn()
  }),
  usePathname: () => '/',
  useSearchParams: () => new URLSearchParams()
}))

// Mock framer-motion for stable testing
vi.mock('framer-motion', () => ({
  motion: {
    div: (props: any) => props.children || null,
    span: (props: any) => props.children || null,
    button: (props: any) => props.children || null,
    section: (props: any) => props.children || null
  },
  AnimatePresence: ({ children }: any) => children,
  useAnimation: () => ({
    start: vi.fn(),
    stop: vi.fn(),
    set: vi.fn()
  })
}))

// Global test utilities
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn()
}))

// File API mocks for drag-drop testing
Object.defineProperty(window, 'File', {
  value: class MockFile {
    constructor(public parts: any[], public name: string, public options: any = {}) {}
    get size() { return 1024 }
    get type() { return this.options.type || 'text/plain' }
  }
})

Object.defineProperty(window, 'DataTransfer', {
  value: class MockDataTransfer {
    files: File[] = []
    items: any[] = []
    types: string[] = []
    
    setData(format: string, data: string) {
      this.types.push(format)
    }
    
    getData(format: string) {
      return ''
    }
  }
})

// Console mocks to reduce noise in tests
global.console = {
  ...console,
  warn: vi.fn(),
  error: vi.fn()
}