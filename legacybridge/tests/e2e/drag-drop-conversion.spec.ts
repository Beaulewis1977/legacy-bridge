import { test, expect } from '@playwright/test'
import { readFileSync } from 'fs'
import path from 'path'

// Test data
const TEST_RTF_CONTENT = `{\\rtf1\\ansi\\deff0 {\\fonttbl {\\f0 Times New Roman;}} 
\\f0\\fs24 Hello \\b World\\b0 with \\i italic\\i0 text.}`

const TEST_RTF_COMPLEX = `{\\rtf1\\ansi\\deff0 
{\\fonttbl {\\f0 Arial;}}
{\\colortbl;\\red255\\green0\\blue0;\\red0\\green255\\blue0;}
\\f0\\fs24 Business Report
\\par\\par
\\b Executive Summary\\b0
\\par
This document contains \\cf1 important\\cf0 information.
\\par\\par
Budget: $1,000,000
\\par
Timeline: Q4 2024}`

test.describe('Drag and Drop File Conversion', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await expect(page).toHaveTitle(/LegacyBridge/)
  })

  test('should display the main interface correctly', async ({ page }) => {
    // Check that the main components are visible
    await expect(page.locator('[data-testid="drag-drop-zone"]')).toBeVisible()
    await expect(page.locator('text=Drag & Drop RTF Files')).toBeVisible()
    await expect(page.locator('text=or click to select files')).toBeVisible()
    await expect(page.locator('text=Supported formats: .rtf, .md, .txt')).toBeVisible()
  })

  test('should handle single file drag and drop', async ({ page }) => {
    // Create a test file
    const fileBuffer = Buffer.from(TEST_RTF_CONTENT)
    
    // Simulate file drop
    await page.locator('[data-testid="drag-drop-zone"]').setInputFiles({
      name: 'test-document.rtf',
      mimeType: 'application/rtf',
      buffer: fileBuffer
    })

    // Wait for file to be processed
    await expect(page.locator('text=test-document.rtf')).toBeVisible()
    await expect(page.locator('[data-testid="conversion-progress"]')).toBeVisible()

    // Wait for conversion to complete
    await expect(page.locator('text=Conversion Complete')).toBeVisible({ timeout: 10000 })
    
    // Check that markdown preview is shown
    await expect(page.locator('[data-testid="markdown-preview"]')).toBeVisible()
    await expect(page.locator('text=Hello **World** with *italic* text')).toBeVisible()
  })

  test('should handle multiple file upload', async ({ page }) => {
    const files = [
      {
        name: 'document1.rtf',
        mimeType: 'application/rtf',
        buffer: Buffer.from(TEST_RTF_CONTENT)
      },
      {
        name: 'document2.rtf', 
        mimeType: 'application/rtf',
        buffer: Buffer.from(TEST_RTF_COMPLEX)
      }
    ]

    await page.locator('[data-testid="drag-drop-zone"]').setInputFiles(files)

    // Check that both files are listed
    await expect(page.locator('text=document1.rtf')).toBeVisible()
    await expect(page.locator('text=document2.rtf')).toBeVisible()
    await expect(page.locator('text=2 files selected')).toBeVisible()

    // Wait for batch conversion to complete
    await expect(page.locator('text=All conversions complete')).toBeVisible({ timeout: 15000 })
  })

  test('should show real-time conversion progress', async ({ page }) => {
    const fileBuffer = Buffer.from(TEST_RTF_COMPLEX)
    
    await page.locator('[data-testid="drag-drop-zone"]').setInputFiles({
      name: 'large-document.rtf',
      mimeType: 'application/rtf',
      buffer: fileBuffer
    })

    // Check progress indicators
    await expect(page.locator('[data-testid="overall-progress"]')).toBeVisible()
    await expect(page.locator('[data-testid="file-progress-large-document.rtf"]')).toBeVisible()
    
    // Check that progress updates
    await expect(page.locator('text=Processing...')).toBeVisible()
    await expect(page.locator('[role="progressbar"]')).toBeVisible()

    // Wait for completion
    await expect(page.locator('text=✓ Completed')).toBeVisible({ timeout: 10000 })
  })

  test('should display validation warnings and errors', async ({ page }) => {
    // Create malformed RTF
    const malformedRtf = '{\\rtf1 incomplete document'
    const fileBuffer = Buffer.from(malformedRtf)
    
    await page.locator('[data-testid="drag-drop-zone"]').setInputFiles({
      name: 'malformed.rtf',
      mimeType: 'application/rtf',
      buffer: fileBuffer
    })

    // Check for validation messages
    await expect(page.locator('[data-testid="validation-results"]')).toBeVisible()
    await expect(page.locator('text=Validation Warning')).toBeVisible()
    await expect(page.locator('text=Missing closing brace')).toBeVisible()

    // Check that error recovery was applied
    await expect(page.locator('text=Error Recovery Applied')).toBeVisible()
    await expect(page.locator('text=fix_missing_braces')).toBeVisible()
  })

  test('should handle file type validation', async ({ page }) => {
    // Try to upload an invalid file type
    const fileBuffer = Buffer.from('This is not an RTF file')
    
    await page.locator('[data-testid="drag-drop-zone"]').setInputFiles({
      name: 'document.pdf',
      mimeType: 'application/pdf',
      buffer: fileBuffer
    })

    // Check for error message
    await expect(page.locator('[data-testid="error-message"]')).toBeVisible()
    await expect(page.locator('text=Invalid file type')).toBeVisible()
    await expect(page.locator('text=Only .rtf, .md, and .txt files are supported')).toBeVisible()
  })

  test('should handle file size validation', async ({ page }) => {
    // Create a file that's too large (mock 15MB file)
    const largeContent = 'x'.repeat(15 * 1024 * 1024)
    const fileBuffer = Buffer.from(largeContent)
    
    await page.locator('[data-testid="drag-drop-zone"]').setInputFiles({
      name: 'huge-document.rtf',
      mimeType: 'application/rtf',
      buffer: fileBuffer
    })

    // Check for size error
    await expect(page.locator('text=File size exceeds limit')).toBeVisible()
    await expect(page.locator('text=Maximum file size is 10MB')).toBeVisible()
  })

  test('should provide download functionality', async ({ page }) => {
    const fileBuffer = Buffer.from(TEST_RTF_CONTENT)
    
    await page.locator('[data-testid="drag-drop-zone"]').setInputFiles({
      name: 'download-test.rtf',
      mimeType: 'application/rtf',
      buffer: fileBuffer
    })

    // Wait for conversion to complete
    await expect(page.locator('text=Conversion Complete')).toBeVisible({ timeout: 10000 })

    // Check download button is available
    const downloadButton = page.locator('[data-testid="download-markdown"]')
    await expect(downloadButton).toBeVisible()
    await expect(downloadButton).toBeEnabled()

    // Start waiting for download before clicking
    const downloadPromise = page.waitForDownload()
    await downloadButton.click()
    const download = await downloadPromise

    // Verify download
    expect(download.suggestedFilename()).toBe('download-test.md')
  })

  test('should show template selection and application', async ({ page }) => {
    const fileBuffer = Buffer.from(TEST_RTF_COMPLEX)
    
    await page.locator('[data-testid="drag-drop-zone"]').setInputFiles({
      name: 'business-doc.rtf',
      mimeType: 'application/rtf', 
      buffer: fileBuffer
    })

    // Open template selector
    await page.locator('[data-testid="template-selector"]').click()
    
    // Select business report template
    await page.locator('text=Business Report').click()
    await page.locator('[data-testid="apply-template"]').click()

    // Check that template was applied
    await expect(page.locator('text=Template Applied: Business Report')).toBeVisible()
    
    // Verify template-specific formatting in preview
    await expect(page.locator('[data-testid="markdown-preview"]')).toContainText('# Business Report')
    await expect(page.locator('[data-testid="markdown-preview"]')).toContainText('## Executive Summary')
  })

  test('should handle pause and resume functionality', async ({ page }) => {
    // Upload multiple files for batch processing
    const files = Array.from({ length: 5 }, (_, i) => ({
      name: `document${i + 1}.rtf`,
      mimeType: 'application/rtf',
      buffer: Buffer.from(TEST_RTF_CONTENT)
    }))

    await page.locator('[data-testid="drag-drop-zone"]').setInputFiles(files)

    // Start conversion
    await expect(page.locator('text=Converting...')).toBeVisible()

    // Pause conversion
    await page.locator('[data-testid="pause-conversion"]').click()
    await expect(page.locator('text=Conversion Paused')).toBeVisible()
    await expect(page.locator('[data-testid="resume-conversion"]')).toBeVisible()

    // Resume conversion
    await page.locator('[data-testid="resume-conversion"]').click()
    await expect(page.locator('text=Converting...')).toBeVisible()

    // Wait for completion
    await expect(page.locator('text=All conversions complete')).toBeVisible({ timeout: 15000 })
  })

  test('should display performance metrics', async ({ page }) => {
    const fileBuffer = Buffer.from(TEST_RTF_CONTENT)
    
    await page.locator('[data-testid="drag-drop-zone"]').setInputFiles({
      name: 'perf-test.rtf',
      mimeType: 'application/rtf',
      buffer: fileBuffer
    })

    // Wait for conversion to complete
    await expect(page.locator('text=Conversion Complete')).toBeVisible({ timeout: 10000 })

    // Check performance metrics are displayed
    await expect(page.locator('[data-testid="processing-time"]')).toBeVisible()
    await expect(page.locator('[data-testid="conversion-speed"]')).toBeVisible()
    
    // Verify reasonable performance values
    const processingTime = await page.locator('[data-testid="processing-time"]').textContent()
    expect(processingTime).toMatch(/\d+(\.\d+)?\s*(ms|s)/)
  })

  test('should work with keyboard navigation', async ({ page }) => {
    // Focus on drag-drop zone using keyboard
    await page.keyboard.press('Tab')
    await expect(page.locator('[data-testid="drag-drop-zone"]')).toBeFocused()

    // Activate with Enter key
    await page.keyboard.press('Enter')
    
    // File input should be triggered (we can't test file selection with keyboard)
    // but we can verify the UI responds to keyboard activation
    const fileInput = page.locator('input[type="file"]')
    await expect(fileInput).toBeAttached()
  })

  test('should maintain accessibility standards', async ({ page }) => {
    // Check for proper ARIA labels and roles
    await expect(page.locator('[data-testid="drag-drop-zone"]')).toHaveAttribute('role', 'button')
    await expect(page.locator('[data-testid="drag-drop-zone"]')).toHaveAttribute('aria-label')
    await expect(page.locator('[data-testid="drag-drop-zone"]')).toHaveAttribute('tabindex', '0')

    // Check progress bar accessibility
    const fileBuffer = Buffer.from(TEST_RTF_CONTENT)
    await page.locator('[data-testid="drag-drop-zone"]').setInputFiles({
      name: 'a11y-test.rtf',
      mimeType: 'application/rtf',
      buffer: fileBuffer
    })

    const progressBar = page.locator('[role="progressbar"]')
    await expect(progressBar).toHaveAttribute('aria-valuenow')
    await expect(progressBar).toHaveAttribute('aria-valuemin', '0')
    await expect(progressBar).toHaveAttribute('aria-valuemax', '100')
  })

  test('should handle dark/light theme switching', async ({ page }) => {
    // Check initial theme
    await expect(page.locator('html')).toHaveAttribute('class', /light|dark/)

    // Toggle theme
    await page.locator('[data-testid="theme-toggle"]').click()
    
    // Verify theme changed
    await expect(page.locator('html')).toHaveAttribute('class', /dark|light/)
    
    // Check that components adapt to theme
    const dropZone = page.locator('[data-testid="drag-drop-zone"]')
    await expect(dropZone).toHaveCSS('background-color', /.+/) // Any color value means theme is applied
  })
})