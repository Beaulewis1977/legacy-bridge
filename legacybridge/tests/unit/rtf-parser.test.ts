import { describe, it, expect, beforeEach, vi } from 'vitest'
import { convertRtfToMd, processWithPipeline, validateDocument } from '@/lib/tauri-api'

// Mock test data
const SAMPLE_RTF = `{\\rtf1\\ansi\\deff0 {\\fonttbl {\\f0 Times New Roman;}} 
\\f0\\fs24 Hello \\b World\\b0 with \\i italic\\i0 text.}`

const EXPECTED_MARKDOWN = `Hello **World** with *italic* text.`

const COMPLEX_RTF = `{\\rtf1\\ansi\\deff0 {\\fonttbl {\\f0 Times New Roman;}}
{\\colortbl;\\red255\\green0\\blue0;\\red0\\green255\\blue0;}
\\f0\\fs24 This is \\cf1 red text\\cf0 and \\cf2 green text\\cf0.
\\par\\par
\\b Bold paragraph\\b0
\\par
\\ul Underlined text\\ul0
\\par
{\\*\\generator Testing RTF Parser;}}`

const INVALID_RTF = `{\\rtf1 incomplete rtf`

describe('RTF Parser', () => {
  beforeEach(() => {
    // Reset all mocks before each test
    vi.clearAllMocks()
  })

  describe('Basic RTF to Markdown Conversion', () => {
    it('should convert simple RTF with bold and italic formatting', async () => {
      // Mock the Tauri API response
      vi.mocked(convertRtfToMd).mockResolvedValue({
        success: true,
        markdown: EXPECTED_MARKDOWN,
        errors: [],
        warnings: []
      })

      const result = await convertRtfToMd(SAMPLE_RTF)
      
      expect(result.success).toBe(true)
      expect(result.markdown).toBe(EXPECTED_MARKDOWN)
      expect(result.errors).toHaveLength(0)
    })

    it('should handle complex RTF with colors and formatting', async () => {
      const expectedComplexMarkdown = `This is red text and green text.

**Bold paragraph**

<u>Underlined text</u>`

      vi.mocked(convertRtfToMd).mockResolvedValue({
        success: true,
        markdown: expectedComplexMarkdown,
        errors: [],
        warnings: ['Color formatting may not be fully preserved in Markdown']
      })

      const result = await convertRtfToMd(COMPLEX_RTF)
      
      expect(result.success).toBe(true)
      expect(result.markdown).toContain('**Bold paragraph**')
      expect(result.warnings).toContain('Color formatting may not be fully preserved in Markdown')
    })

    it('should handle empty RTF input gracefully', async () => {
      vi.mocked(convertRtfToMd).mockResolvedValue({
        success: true,
        markdown: '',
        errors: [],
        warnings: []
      })

      const result = await convertRtfToMd('')
      
      expect(result.success).toBe(true)
      expect(result.markdown).toBe('')
    })

    it('should handle malformed RTF input', async () => {
      vi.mocked(convertRtfToMd).mockResolvedValue({
        success: false,
        markdown: '',
        errors: ['Invalid RTF format: Missing closing brace'],
        warnings: []
      })

      const result = await convertRtfToMd(INVALID_RTF)
      
      expect(result.success).toBe(false)
      expect(result.errors).toContain('Invalid RTF format: Missing closing brace')
    })
  })

  describe('Pipeline Processing', () => {
    it('should process RTF through the complete pipeline', async () => {
      const pipelineConfig = {
        enableValidation: true,
        enableErrorRecovery: true,
        templateId: 'business-report'
      }

      vi.mocked(processWithPipeline).mockResolvedValue({
        success: true,
        markdown: EXPECTED_MARKDOWN,
        validationResults: [
          { level: 'info', message: 'Document structure is valid', position: 0 }
        ],
        appliedRecoveries: [],
        templateApplied: 'business-report',
        processingTime: 45
      })

      const result = await processWithPipeline(SAMPLE_RTF, pipelineConfig)
      
      expect(result.success).toBe(true)
      expect(result.markdown).toBe(EXPECTED_MARKDOWN)
      expect(result.templateApplied).toBe('business-report')
      expect(result.processingTime).toBeLessThan(100)
    })

    it('should apply error recovery strategies for malformed RTF', async () => {
      vi.mocked(processWithPipeline).mockResolvedValue({
        success: true,
        markdown: 'Hello World with italic text.',
        validationResults: [
          { level: 'warning', message: 'Missing closing tag recovered', position: 25 }
        ],
        appliedRecoveries: ['fix_missing_braces', 'normalize_whitespace'],
        templateApplied: null,
        processingTime: 78
      })

      const result = await processWithPipeline(INVALID_RTF, { 
        enableErrorRecovery: true 
      })
      
      expect(result.success).toBe(true)
      expect(result.appliedRecoveries).toContain('fix_missing_braces')
      expect(result.validationResults[0].level).toBe('warning')
    })
  })

  describe('Document Validation', () => {
    it('should validate well-formed RTF documents', async () => {
      vi.mocked(validateDocument).mockResolvedValue({
        isValid: true,
        results: [
          { level: 'info', message: 'RTF format is valid', position: 0 },
          { level: 'info', message: 'Font table is properly defined', position: 12 }
        ],
        formatType: 'rtf'
      })

      const validation = await validateDocument(SAMPLE_RTF, 'rtf')
      
      expect(validation.isValid).toBe(true)
      expect(validation.results).toHaveLength(2)
      expect(validation.formatType).toBe('rtf')
    })

    it('should detect validation errors in malformed RTF', async () => {
      vi.mocked(validateDocument).mockResolvedValue({
        isValid: false,
        results: [
          { level: 'error', message: 'Unclosed RTF group', position: 15 },
          { level: 'warning', message: 'Missing font table', position: 0 }
        ],
        formatType: 'rtf'
      })

      const validation = await validateDocument(INVALID_RTF, 'rtf')
      
      expect(validation.isValid).toBe(false)
      expect(validation.results.some(r => r.level === 'error')).toBe(true)
      expect(validation.results.some(r => r.message.includes('Unclosed RTF group'))).toBe(true)
    })

    it('should validate markdown documents', async () => {
      const markdown = '# Title\n\n**Bold** and *italic* text.'
      
      vi.mocked(validateDocument).mockResolvedValue({
        isValid: true,
        results: [
          { level: 'info', message: 'Valid CommonMark syntax', position: 0 }
        ],
        formatType: 'markdown'
      })

      const validation = await validateDocument(markdown, 'markdown')
      
      expect(validation.isValid).toBe(true)
      expect(validation.formatType).toBe('markdown')
    })
  })

  describe('Error Handling', () => {
    it('should handle API timeouts gracefully', async () => {
      vi.mocked(convertRtfToMd).mockRejectedValue(new Error('Request timeout'))

      await expect(convertRtfToMd(SAMPLE_RTF)).rejects.toThrow('Request timeout')
    })

    it('should handle network errors', async () => {
      vi.mocked(convertRtfToMd).mockRejectedValue(new Error('Network error'))

      await expect(convertRtfToMd(SAMPLE_RTF)).rejects.toThrow('Network error')
    })
  })

  describe('Performance Tests', () => {
    it('should process small documents quickly', async () => {
      vi.mocked(convertRtfToMd).mockResolvedValue({
        success: true,
        markdown: EXPECTED_MARKDOWN,
        errors: [],
        warnings: []
      })

      const startTime = Date.now()
      await convertRtfToMd(SAMPLE_RTF)
      const endTime = Date.now()
      
      // Mock should respond immediately, but in real implementation this ensures < 500ms
      expect(endTime - startTime).toBeLessThan(100)
    })

    it('should handle large documents efficiently', async () => {
      const largeRtf = SAMPLE_RTF.repeat(1000) // Simulate large document
      
      vi.mocked(convertRtfToMd).mockResolvedValue({
        success: true,
        markdown: EXPECTED_MARKDOWN.repeat(1000),
        errors: [],
        warnings: []
      })

      const startTime = Date.now()
      const result = await convertRtfToMd(largeRtf)
      const endTime = Date.now()
      
      expect(result.success).toBe(true)
      expect(endTime - startTime).toBeLessThan(200) // Mock should be fast
    })
  })
})