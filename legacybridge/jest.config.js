module.exports = {
  // Test environment
  testEnvironment: 'jsdom',
  
  // Setup files
  setupFilesAfterEnv: ['<rootDir>/tests/setup/jest.setup.js'],
  
  // Module paths
  moduleNameMapper: {
    '^@/(.*)$': '<rootDir>/src/$1',
    '^@components/(.*)$': '<rootDir>/src/components/$1',
    '^@lib/(.*)$': '<rootDir>/src/lib/$1',
    '^@utils/(.*)$': '<rootDir>/src/utils/$1',
    // Handle CSS imports
    '\\.(css|less|sass|scss)$': '<rootDir>/tests/mocks/styleMock.js',
    // Handle image imports
    '\\.(jpg|jpeg|png|gif|svg)$': '<rootDir>/tests/mocks/fileMock.js'
  },
  
  // Transform files
  transform: {
    '^.+\\.(js|jsx|ts|tsx)$': ['ts-jest', {
      tsconfig: {
        jsx: 'react-jsx'
      }
    }]
  },
  
  // Test patterns
  testMatch: [
    '<rootDir>/tests/**/*.{test,spec}.{js,jsx,ts,tsx}',
    '<rootDir>/src/**/__tests__/**/*.{js,jsx,ts,tsx}'
  ],
  
  // Coverage configuration
  collectCoverage: true,
  collectCoverageFrom: [
    'src/**/*.{js,jsx,ts,tsx}',
    '!src/**/*.d.ts',
    '!src/**/*.stories.{js,jsx,ts,tsx}',
    '!src/**/index.{js,ts}',
    '!src/types/**',
    '!src/**/__tests__/**',
    '!src/**/__mocks__/**'
  ],
  coverageDirectory: 'coverage',
  coverageReporters: ['text', 'lcov', 'html', 'json'],
  coverageThreshold: {
    global: {
      branches: 95,
      functions: 95,
      lines: 95,
      statements: 95
    }
  },
  
  // Module directories
  moduleDirectories: ['node_modules', 'src'],
  
  // File extensions
  moduleFileExtensions: ['js', 'jsx', 'ts', 'tsx', 'json'],
  
  // Ignore patterns
  testPathIgnorePatterns: [
    '/node_modules/',
    '/build/',
    '/dist/',
    '/.next/',
    '/out/'
  ],
  
  // Globals
  globals: {
    'ts-jest': {
      isolatedModules: true
    }
  },
  
  // Timers
  timers: 'modern',
  
  // Verbose output
  verbose: true,
  
  // Max workers
  maxWorkers: '50%',
  
  // Test timeout
  testTimeout: 30000,
  
  // Watch plugins
  watchPlugins: [
    'jest-watch-typeahead/filename',
    'jest-watch-typeahead/testname'
  ]
};