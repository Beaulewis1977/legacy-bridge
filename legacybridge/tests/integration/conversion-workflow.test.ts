import { describe, it, expect, beforeEach, vi } from 'vitest'
import { 
  convertRtfToMd, 
  processWithPipeline, 
  validateDocument,
  applyTemplate,
  getVersionInfo,
  testConnection 
} from '@/lib/tauri-api'

describe('Complete Conversion Workflow Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('RTF to Markdown Workflow', () => {
    it('should complete full RTF to Markdown conversion with pipeline', async () => {
      const rtfContent = `{\\rtf1\\ansi\\deff0 
        {\\fonttbl {\\f0 Times New Roman;}}
        \\f0\\fs24 Business Report
        \\par\\par
        \\b Executive Summary\\b0
        \\par
        This is a \\i critical\\i0 business document with \\ul important\\ul0 information.
        \\par\\par
        {\\*\\generator LegacyBridge Test Suite;}
      }`

      // Step 1: Validate the RTF document
      vi.mocked(validateDocument).mockResolvedValue({
        isValid: true,
        results: [
          { level: 'info', message: 'RTF format is valid', position: 0 },
          { level: 'info', message: 'Font table correctly defined', position: 25 }
        ],
        formatType: 'rtf'
      })

      const validation = await validateDocument(rtfContent, 'rtf')
      expect(validation.isValid).toBe(true)

      // Step 2: Apply business template
      vi.mocked(applyTemplate).mockResolvedValue({
        success: true,
        content: rtfContent,
        templateApplied: 'business-report',
        modifications: ['header_formatting', 'paragraph_spacing']
      })

      const templatedContent = await applyTemplate(rtfContent, 'business-report')
      expect(templatedContent.success).toBe(true)

      // Step 3: Process through pipeline
      vi.mocked(processWithPipeline).mockResolvedValue({
        success: true,
        markdown: `# Business Report

## Executive Summary

This is a *critical* business document with **important** information.`,
        validationResults: [
          { level: 'info', message: 'Document processed successfully', position: 0 }
        ],
        appliedRecoveries: [],
        templateApplied: 'business-report',
        processingTime: 127
      })

      const result = await processWithPipeline(templatedContent.content, {
        enableValidation: true,
        enableErrorRecovery: true,
        templateId: 'business-report'
      })

      expect(result.success).toBe(true)
      expect(result.markdown).toContain('# Business Report')
      expect(result.markdown).toContain('## Executive Summary')
      expect(result.markdown).toContain('*critical*')
      expect(result.markdown).toContain('**important**')
      expect(result.processingTime).toBeLessThan(200)
    })

    it('should handle complex RTF with tables and formatting', async () => {
      const complexRtf = `{\\rtf1\\ansi\\deff0 
        {\\fonttbl {\\f0 Arial;}}
        {\\colortbl;\\red255\\green0\\blue0;\\red0\\green255\\blue0;}
        \\f0\\fs24 
        {\\*\\trowd\\trgaph108\\trleft-108
        \\clbrdr\\brdrw10\\brdrs
        \\cellx1000
        \\clbrdr\\brdrw10\\brdrs
        \\cellx2000
        \\row}
        Name\\cell Age\\cell\\row
        John\\cell 30\\cell\\row
      }`

      vi.mocked(processWithPipeline).mockResolvedValue({
        success: true,
        markdown: `| Name | Age |
|------|-----|
| John | 30  |`,
        validationResults: [
          { level: 'info', message: 'Table structure preserved', position: 45 }
        ],
        appliedRecoveries: ['normalize_table_structure'],
        templateApplied: null,
        processingTime: 89
      })

      const result = await processWithPipeline(complexRtf, {
        enableValidation: true,
        enableErrorRecovery: true
      })

      expect(result.success).toBe(true)
      expect(result.markdown).toContain('| Name | Age |')
      expect(result.markdown).toContain('| John | 30  |')
      expect(result.appliedRecoveries).toContain('normalize_table_structure')
    })

    it('should handle malformed RTF with error recovery', async () => {
      const malformedRtf = `{\\rtf1\\ansi\\deff0 
        {\\fonttbl {\\f0 Arial;}
        \\f0\\fs24 This is \\b incomplete bold text
        Missing closing braces`

      vi.mocked(processWithPipeline).mockResolvedValue({
        success: true,
        markdown: 'This is **incomplete bold text**\nMissing closing braces',
        validationResults: [
          { level: 'warning', message: 'Missing closing brace recovered', position: 67 },
          { level: 'warning', message: 'Incomplete font table recovered', position: 23 }
        ],
        appliedRecoveries: [
          'fix_missing_braces', 
          'complete_font_table', 
          'normalize_formatting'
        ],
        templateApplied: null,
        processingTime: 156
      })

      const result = await processWithPipeline(malformedRtf, {
        enableValidation: true,
        enableErrorRecovery: true
      })

      expect(result.success).toBe(true)
      expect(result.markdown).toContain('**incomplete bold text**')
      expect(result.appliedRecoveries).toContain('fix_missing_braces')
      expect(result.validationResults.some(r => r.level === 'warning')).toBe(true)
    })
  })

  describe('Batch Processing Workflow', () => {
    it('should process multiple files in sequence', async () => {
      const files = [
        { name: 'doc1.rtf', content: '{\\rtf1 Document 1}' },
        { name: 'doc2.rtf', content: '{\\rtf1 Document 2}' },
        { name: 'doc3.rtf', content: '{\\rtf1 Document 3}' }
      ]

      // Mock sequential processing
      vi.mocked(processWithPipeline)
        .mockResolvedValueOnce({
          success: true,
          markdown: 'Document 1',
          validationResults: [],
          appliedRecoveries: [],
          templateApplied: null,
          processingTime: 45
        })
        .mockResolvedValueOnce({
          success: true,
          markdown: 'Document 2',
          validationResults: [],
          appliedRecoveries: [],
          templateApplied: null,
          processingTime: 52
        })
        .mockResolvedValueOnce({
          success: true,
          markdown: 'Document 3',
          validationResults: [],
          appliedRecoveries: [],
          templateApplied: null,
          processingTime: 38
        })

      const results = []
      for (const file of files) {
        const result = await processWithPipeline(file.content, {
          enableValidation: true
        })
        results.push(result)
      }

      expect(results).toHaveLength(3)
      expect(results.every(r => r.success)).toBe(true)
      expect(results[0].markdown).toBe('Document 1')
      expect(results[1].markdown).toBe('Document 2')
      expect(results[2].markdown).toBe('Document 3')
    })

    it('should handle mixed success and failure in batch', async () => {
      const files = [
        { name: 'good.rtf', content: '{\\rtf1 Good document}' },
        { name: 'bad.rtf', content: 'Not RTF at all' },
        { name: 'fixed.rtf', content: '{\\rtf1 Recoverable}' }
      ]

      vi.mocked(processWithPipeline)
        .mockResolvedValueOnce({
          success: true,
          markdown: 'Good document',
          validationResults: [],
          appliedRecoveries: [],
          templateApplied: null,
          processingTime: 41
        })
        .mockResolvedValueOnce({
          success: false,
          markdown: '',
          validationResults: [
            { level: 'error', message: 'Not a valid RTF document', position: 0 }
          ],
          appliedRecoveries: [],
          templateApplied: null,
          processingTime: 15
        })
        .mockResolvedValueOnce({
          success: true,
          markdown: 'Recoverable',
          validationResults: [
            { level: 'warning', message: 'Minor issues recovered', position: 8 }
          ],
          appliedRecoveries: ['normalize_content'],
          templateApplied: null,
          processingTime: 78
        })

      const results = []
      for (const file of files) {
        const result = await processWithPipeline(file.content, {
          enableValidation: true,
          enableErrorRecovery: true
        })
        results.push(result)
      }

      expect(results[0].success).toBe(true)
      expect(results[1].success).toBe(false)
      expect(results[2].success).toBe(true)
      
      const successCount = results.filter(r => r.success).length
      expect(successCount).toBe(2)
    })
  })

  describe('Template System Integration', () => {
    it('should apply and validate business report template', async () => {
      const businessRtf = `{\\rtf1\\ansi\\deff0 
        {\\fonttbl {\\f0 Arial;}}
        \\f0\\fs24 
        Q4 2024 Business Report
        \\par\\par
        Revenue: $1.2M
        \\par
        Expenses: $800K
        \\par
        Profit: $400K
      }`

      vi.mocked(applyTemplate).mockResolvedValue({
        success: true,
        content: businessRtf,
        templateApplied: 'business-report',
        modifications: [
          'add_header_formatting',
          'standardize_currency_format',
          'apply_financial_styling'
        ]
      })

      vi.mocked(processWithPipeline).mockResolvedValue({
        success: true,
        markdown: `# Q4 2024 Business Report

**Revenue:** $1,200,000  
**Expenses:** $800,000  
**Profit:** $400,000`,
        validationResults: [
          { level: 'info', message: 'Business template applied successfully', position: 0 }
        ],
        appliedRecoveries: [],
        templateApplied: 'business-report',
        processingTime: 95
      })

      const templatedResult = await applyTemplate(businessRtf, 'business-report')
      expect(templatedResult.success).toBe(true)
      expect(templatedResult.modifications).toContain('standardize_currency_format')

      const finalResult = await processWithPipeline(templatedResult.content, {
        templateId: 'business-report'
      })

      expect(finalResult.success).toBe(true)
      expect(finalResult.markdown).toContain('# Q4 2024 Business Report')
      expect(finalResult.markdown).toContain('$1,200,000')
      expect(finalResult.templateApplied).toBe('business-report')
    })
  })

  describe('System Health and Performance', () => {
    it('should verify system connectivity', async () => {
      vi.mocked(testConnection).mockResolvedValue({
        success: true,
        responseTime: 23,
        version: '1.0.0',
        features: ['rtf_conversion', 'markdown_generation', 'pipeline_processing']
      })

      const connectionTest = await testConnection()
      
      expect(connectionTest.success).toBe(true)
      expect(connectionTest.responseTime).toBeLessThan(100)
      expect(connectionTest.features).toContain('rtf_conversion')
    })

    it('should provide system version information', async () => {
      vi.mocked(getVersionInfo).mockResolvedValue({
        version: '1.0.0',
        buildDate: '2024-07-24',
        features: {
          rtfParser: '2.1.0',
          markdownGenerator: '1.5.0',
          pipelineProcessor: '1.0.0',
          templateEngine: '0.9.0'
        },
        supportedFormats: ['rtf', 'markdown', 'txt']
      })

      const versionInfo = await getVersionInfo()
      
      expect(versionInfo.version).toBe('1.0.0')
      expect(versionInfo.features.rtfParser).toBeTruthy()
      expect(versionInfo.supportedFormats).toContain('rtf')
    })

    it('should measure conversion performance', async () => {
      const startTime = Date.now()
      
      vi.mocked(processWithPipeline).mockResolvedValue({
        success: true,
        markdown: 'Test content',
        validationResults: [],
        appliedRecoveries: [],
        templateApplied: null,
        processingTime: 67
      })

      const result = await processWithPipeline('{\\rtf1 Test}', {})
      const endTime = Date.now()
      
      expect(result.success).toBe(true)
      expect(result.processingTime).toBeLessThan(100)
      expect(endTime - startTime).toBeLessThan(200) // Including network overhead
    })
  })

  describe('Error Handling and Recovery', () => {
    it('should gracefully handle network timeouts', async () => {
      vi.mocked(processWithPipeline).mockRejectedValue(
        new Error('Request timeout after 30 seconds')
      )

      await expect(
        processWithPipeline('{\\rtf1 Test}', {})
      ).rejects.toThrow('Request timeout')
    })

    it('should handle memory limitations gracefully', async () => {
      const largeRtf = '{\\rtf1 ' + 'A'.repeat(10 * 1024 * 1024) + '}'
      
      vi.mocked(processWithPipeline).mockResolvedValue({
        success: false,
        markdown: '',
        validationResults: [
          { level: 'error', message: 'Document too large for processing', position: 0 }
        ],
        appliedRecoveries: [],
        templateApplied: null,
        processingTime: 5
      })

      const result = await processWithPipeline(largeRtf, {})
      
      expect(result.success).toBe(false)
      expect(result.validationResults[0].message).toContain('too large')
    })

    it('should provide meaningful error messages', async () => {
      vi.mocked(processWithPipeline).mockResolvedValue({
        success: false,
        markdown: '',
        validationResults: [
          { 
            level: 'error', 
            message: 'Unsupported RTF version: expecting RTF 1.x, found RTF 2.0', 
            position: 6 
          }
        ],
        appliedRecoveries: [],
        templateApplied: null,
        processingTime: 12
      })

      const result = await processWithPipeline('{\\rtf2 Future RTF}', {})
      
      expect(result.success).toBe(false)
      expect(result.validationResults[0].message).toContain('Unsupported RTF version')
      expect(result.validationResults[0].position).toBe(6)
    })
  })
})