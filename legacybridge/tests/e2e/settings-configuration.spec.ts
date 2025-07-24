import { test, expect } from '@playwright/test'

test.describe('Settings and Configuration', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await expect(page).toHaveTitle(/LegacyBridge/)
  })

  test('should open and close settings panel', async ({ page }) => {
    // Open settings panel
    await page.locator('[data-testid="settings-button"]').click()
    await expect(page.locator('[data-testid="settings-panel"]')).toBeVisible()
    await expect(page.locator('text=Conversion Settings')).toBeVisible()

    // Close settings panel
    await page.locator('[data-testid="close-settings"]').click()
    await expect(page.locator('[data-testid="settings-panel"]')).not.toBeVisible()
  })

  test('should configure validation settings', async ({ page }) => {
    await page.locator('[data-testid="settings-button"]').click()
    
    // Toggle validation settings
    const enableValidation = page.locator('[data-testid="enable-validation"]')
    await expect(enableValidation).toBeVisible()
    
    // Check initial state
    const isChecked = await enableValidation.isChecked()
    
    // Toggle the setting
    await enableValidation.click()
    await expect(enableValidation).toBeChecked(!isChecked)

    // Check strict validation option appears when validation is enabled
    if (!isChecked) {
      await expect(page.locator('[data-testid="strict-validation"]')).toBeVisible()
    }
  })

  test('should configure error recovery settings', async ({ page }) => {
    await page.locator('[data-testid="settings-button"]').click()
    
    // Enable error recovery
    const enableRecovery = page.locator('[data-testid="enable-error-recovery"]')
    await enableRecovery.check()
    await expect(enableRecovery).toBeChecked()

    // Configure recovery strategies
    await expect(page.locator('[data-testid="recovery-strategies"]')).toBeVisible()
    
    const strategies = [
      'fix-missing-braces',
      'normalize-whitespace', 
      'repair-font-table',
      'handle-unknown-codes',
      'best-effort-parsing'
    ]

    for (const strategy of strategies) {
      const checkbox = page.locator(`[data-testid="strategy-${strategy}"]`)
      await expect(checkbox).toBeVisible()
      await checkbox.check()
      await expect(checkbox).toBeChecked()
    }
  })

  test('should configure template settings', async ({ page }) => {
    await page.locator('[data-testid="settings-button"]').click()
    
    // Navigate to template settings
    await page.locator('[data-testid="template-settings-tab"]').click()
    
    // Check available templates
    const templates = [
      'business-report',
      'technical-document',
      'meeting-notes',
      'legal-document',
      'academic-paper'
    ]

    for (const template of templates) {
      await expect(page.locator(`text=${template}`)).toBeVisible()
    }

    // Select default template
    await page.locator('[data-testid="default-template-select"]').click()
    await page.locator('text=Business Report').click()
    
    // Verify selection
    await expect(page.locator('[data-testid="default-template-select"]')).toContainText('Business Report')
  })

  test('should configure output format settings', async ({ page }) => {
    await page.locator('[data-testid="settings-button"]').click()
    
    // Navigate to output settings
    await page.locator('[data-testid="output-settings-tab"]').click()
    
    // Configure markdown flavor
    await page.locator('[data-testid="markdown-flavor-select"]').click()
    await page.locator('text=GitHub Flavored Markdown').click()
    
    // Configure line ending style
    await page.locator('[data-testid="line-endings-select"]').click()
    await page.locator('text=LF (Unix)').click()
    
    // Configure text wrapping
    const wrapAt = page.locator('[data-testid="wrap-at-column"]')
    await wrapAt.fill('80')
    await expect(wrapAt).toHaveValue('80')

    // Configure preserve formatting options
    await page.locator('[data-testid="preserve-tables"]').check()
    await page.locator('[data-testid="preserve-lists"]').check()
    await page.locator('[data-testid="preserve-headers"]').check()
  })

  test('should configure performance settings', async ({ page }) => {
    await page.locator('[data-testid="settings-button"]').click()
    
    // Navigate to performance settings
    await page.locator('[data-testid="performance-settings-tab"]').click()
    
    // Configure batch size
    const batchSize = page.locator('[data-testid="batch-size"]')
    await batchSize.fill('5')
    await expect(batchSize).toHaveValue('5')

    // Configure timeout settings
    const conversionTimeout = page.locator('[data-testid="conversion-timeout"]')
    await conversionTimeout.fill('30')
    await expect(conversionTimeout).toHaveValue('30')

    // Configure memory limit
    const memoryLimit = page.locator('[data-testid="memory-limit"]')
    await memoryLimit.fill('100')
    await expect(memoryLimit).toHaveValue('100')

    // Enable/disable parallel processing
    const parallelProcessing = page.locator('[data-testid="parallel-processing"]')
    await parallelProcessing.check()
    await expect(parallelProcessing).toBeChecked()
  })

  test('should save and load configuration profiles', async ({ page }) => {
    await page.locator('[data-testid="settings-button"]').click()
    
    // Configure some settings
    await page.locator('[data-testid="enable-validation"]').check()
    await page.locator('[data-testid="enable-error-recovery"]').check()
    
    // Save configuration profile
    await page.locator('[data-testid="save-profile-button"]').click()
    
    // Enter profile name
    const profileNameInput = page.locator('[data-testid="profile-name-input"]')
    await profileNameInput.fill('My Custom Profile')
    await page.locator('[data-testid="confirm-save-profile"]').click()
    
    // Verify profile was saved
    await expect(page.locator('text=Profile saved successfully')).toBeVisible()
    
    // Load the profile
    await page.locator('[data-testid="load-profile-select"]').click()
    await page.locator('text=My Custom Profile').click()
    await page.locator('[data-testid="load-profile-button"]').click()
    
    // Verify settings were loaded
    await expect(page.locator('[data-testid="enable-validation"]')).toBeChecked()
    await expect(page.locator('[data-testid="enable-error-recovery"]')).toBeChecked()
  })

  test('should export and import configuration', async ({ page }) => {
    await page.locator('[data-testid="settings-button"]').click()
    
    // Configure some settings
    await page.locator('[data-testid="enable-validation"]').check()
    await page.locator('[data-testid="template-settings-tab"]').click()
    await page.locator('[data-testid="default-template-select"]').click()
    await page.locator('text=Technical Document').click()
    
    // Export configuration
    const downloadPromise = page.waitForDownload()
    await page.locator('[data-testid="export-config"]').click()
    const download = await downloadPromise
    
    expect(download.suggestedFilename()).toBe('legacybridge-config.json')
    
    // Import configuration
    await page.locator('[data-testid="import-config"]').setInputFiles({
      name: 'test-config.json',
      mimeType: 'application/json',
      buffer: Buffer.from(JSON.stringify({
        validation: { enabled: false },
        template: { default: 'business-report' },
        performance: { batchSize: 10 }
      }))
    })
    
    // Verify import success
    await expect(page.locator('text=Configuration imported successfully')).toBeVisible()
    
    // Verify settings were applied
    await expect(page.locator('[data-testid="enable-validation"]')).not.toBeChecked()
  })

  test('should reset settings to defaults', async ({ page }) => {
    await page.locator('[data-testid="settings-button"]').click()
    
    // Change some settings from defaults
    await page.locator('[data-testid="enable-validation"]').check()
    await page.locator('[data-testid="enable-error-recovery"]').check()
    
    // Reset to defaults
    await page.locator('[data-testid="reset-to-defaults"]').click()
    
    // Confirm reset
    await page.locator('[data-testid="confirm-reset"]').click()
    
    // Verify reset success message
    await expect(page.locator('text=Settings reset to defaults')).toBeVisible()
    
    // Verify settings were reset (assuming defaults are unchecked)
    await expect(page.locator('[data-testid="enable-validation"]')).not.toBeChecked()
    await expect(page.locator('[data-testid="enable-error-recovery"]')).not.toBeChecked()
  })

  test('should show settings validation errors', async ({ page }) => {
    await page.locator('[data-testid="settings-button"]').click()
    
    // Navigate to performance settings
    await page.locator('[data-testid="performance-settings-tab"]').click()
    
    // Enter invalid values
    const batchSize = page.locator('[data-testid="batch-size"]')
    await batchSize.fill('0')
    
    const memoryLimit = page.locator('[data-testid="memory-limit"]')
    await memoryLimit.fill('-10')
    
    // Try to save settings
    await page.locator('[data-testid="save-settings"]').click()
    
    // Check for validation errors
    await expect(page.locator('text=Batch size must be at least 1')).toBeVisible()
    await expect(page.locator('text=Memory limit must be positive')).toBeVisible()
    
    // Verify settings weren't saved
    await expect(page.locator('text=Settings contain errors')).toBeVisible()
  })

  test('should show real-time settings preview', async ({ page }) => {
    await page.locator('[data-testid="settings-button"]').click()
    
    // Upload a test file first
    const testRtf = '{\\rtf1 Test document}'
    await page.locator('[data-testid="drag-drop-zone"]').setInputFiles({
      name: 'preview-test.rtf',
      mimeType: 'application/rtf',
      buffer: Buffer.from(testRtf)
    })
    
    // Open settings while file is processing
    await page.locator('[data-testid="settings-button"]').click()
    
    // Change a setting and see preview update
    await page.locator('[data-testid="template-settings-tab"]').click()
    await page.locator('[data-testid="default-template-select"]').click()
    await page.locator('text=Business Report').click()
    
    // Check that preview updates in real-time
    await expect(page.locator('[data-testid="settings-preview"]')).toBeVisible()
    await expect(page.locator('[data-testid="settings-preview"]')).toContainText('# Test Document')
  })

  test('should maintain settings across page reloads', async ({ page }) => {
    await page.locator('[data-testid="settings-button"]').click()
    
    // Configure settings
    await page.locator('[data-testid="enable-validation"]').check()
    await page.locator('[data-testid="template-settings-tab"]').click()
    await page.locator('[data-testid="default-template-select"]').click()
    await page.locator('text=Technical Document').click()
    
    // Save settings
    await page.locator('[data-testid="save-settings"]').click()
    await expect(page.locator('text=Settings saved')).toBeVisible()
    
    // Reload page
    await page.reload()
    await expect(page).toHaveTitle(/LegacyBridge/)
    
    // Open settings and verify they were persisted
    await page.locator('[data-testid="settings-button"]').click()
    await expect(page.locator('[data-testid="enable-validation"]')).toBeChecked()
    
    await page.locator('[data-testid="template-settings-tab"]').click()
    await expect(page.locator('[data-testid="default-template-select"]')).toContainText('Technical Document')
  })

  test('should show settings help and tooltips', async ({ page }) => {
    await page.locator('[data-testid="settings-button"]').click()
    
    // Hover over setting labels to see tooltips
    await page.locator('[data-testid="validation-help"]').hover()
    await expect(page.locator('[data-testid="tooltip"]')).toBeVisible()
    await expect(page.locator('[data-testid="tooltip"]')).toContainText('Validates RTF document structure')
    
    // Click help button for detailed explanation
    await page.locator('[data-testid="template-help"]').click()
    await expect(page.locator('[data-testid="help-modal"]')).toBeVisible()
    await expect(page.locator('[data-testid="help-modal"]')).toContainText('Templates apply consistent formatting')
    
    // Close help modal
    await page.locator('[data-testid="close-help"]').click()
    await expect(page.locator('[data-testid="help-modal"]')).not.toBeVisible()
  })
})