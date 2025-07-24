// Jest setup file for global test configuration
import '@testing-library/jest-dom';

// Mock Tauri API for tests
global.__TAURI__ = {
  invoke: jest.fn(),
  event: {
    emit: jest.fn(),
    listen: jest.fn(),
    once: jest.fn(),
    unlisten: jest.fn()
  },
  window: {
    appWindow: {
      minimize: jest.fn(),
      maximize: jest.fn(),
      close: jest.fn(),
      setTitle: jest.fn()
    }
  },
  fs: {
    readFile: jest.fn(),
    writeFile: jest.fn(),
    exists: jest.fn()
  },
  path: {
    appDataDir: jest.fn(() => Promise.resolve('/app/data')),
    join: jest.fn((...parts) => parts.join('/'))
  }
};

// Global test utilities
global.testUtils = {
  // Generate test RTF content
  generateRTF: (size = 1024) => {
    const header = '{\\rtf1\\ansi\\deff0 {\\fonttbl{\\f0 Times New Roman;}}';
    const footer = '}';
    const content = '\\f0\\fs24 Test content '.repeat(Math.floor(size / 20));
    return `${header}${content}${footer}`;
  },

  // Generate test Markdown content
  generateMarkdown: (paragraphs = 10) => {
    let content = '# Test Document\\n\\n';
    for (let i = 0; i < paragraphs; i++) {
      content += `Paragraph ${i + 1} with **bold** and *italic* text.\\n\\n`;
    }
    return content;
  },

  // Wait for condition
  waitFor: async (condition, timeout = 5000) => {
    const start = Date.now();
    while (!condition()) {
      if (Date.now() - start > timeout) {
        throw new Error('Timeout waiting for condition');
      }
      await new Promise(resolve => setTimeout(resolve, 100));
    }
  }
};

// Mock window.matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: jest.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: jest.fn(),
    removeListener: jest.fn(),
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
    dispatchEvent: jest.fn(),
  })),
});

// Mock IntersectionObserver
global.IntersectionObserver = class IntersectionObserver {
  constructor() {}
  disconnect() {}
  observe() {}
  unobserve() {}
};

// Mock ResizeObserver
global.ResizeObserver = class ResizeObserver {
  constructor() {}
  disconnect() {}
  observe() {}
  unobserve() {}
};

// Suppress console errors in tests unless explicitly testing them
const originalError = console.error;
beforeAll(() => {
  console.error = (...args) => {
    if (
      typeof args[0] === 'string' &&
      (args[0].includes('Warning: ReactDOM.render') ||
       args[0].includes('Warning: unmountComponentAtNode'))
    ) {
      return;
    }
    originalError.call(console, ...args);
  };
});

afterAll(() => {
  console.error = originalError;
});

// Clean up after each test
afterEach(() => {
  jest.clearAllMocks();
});