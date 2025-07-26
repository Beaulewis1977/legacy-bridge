* LegacyBridge VFP9 Test Program
* Demonstrates usage of the LegacyBridge DLL for MD<->RTF conversion

CLEAR
SET TALK OFF
SET SAFETY OFF

* Create form for testing
DO FORM TestLegacyBridgeForm

* Form definition
DEFINE CLASS TestLegacyBridgeForm AS Form
    Height = 500
    Width = 700
    Caption = "LegacyBridge Test - MD to RTF Converter"
    AutoCenter = .T.
    
    ADD OBJECT lblInput AS Label WITH ;
        Caption = "Input:", ;
        Left = 10, ;
        Top = 10, ;
        Width = 50
        
    ADD OBJECT txtInput AS EditBox WITH ;
        Left = 10, ;
        Top = 30, ;
        Width = 680, ;
        Height = 150, ;
        Value = "{\rtf1\ansi\deff0 {\fonttbl{\f0 Times New Roman;}}" + CHR(13) + CHR(10) + ;
                "\b Hello World\b0\par" + CHR(13) + CHR(10) + ;
                "This is a \i test\i0 document.\par" + CHR(13) + CHR(10) + ;
                "}"
    
    ADD OBJECT lblOutput AS Label WITH ;
        Caption = "Output:", ;
        Left = 10, ;
        Top = 190, ;
        Width = 50
        
    ADD OBJECT txtOutput AS EditBox WITH ;
        Left = 10, ;
        Top = 210, ;
        Width = 680, ;
        Height = 150, ;
        ReadOnly = .T.
        
    ADD OBJECT cmdRtfToMd AS CommandButton WITH ;
        Caption = "RTF → Markdown", ;
        Left = 150, ;
        Top = 370, ;
        Width = 120, ;
        Height = 30
        
    ADD OBJECT cmdMdToRtf AS CommandButton WITH ;
        Caption = "Markdown → RTF", ;
        Left = 280, ;
        Top = 370, ;
        Width = 120, ;
        Height = 30
        
    ADD OBJECT cmdVersion AS CommandButton WITH ;
        Caption = "Get Version", ;
        Left = 410, ;
        Top = 370, ;
        Width = 100, ;
        Height = 30
        
    ADD OBJECT cmdBatch AS CommandButton WITH ;
        Caption = "Batch Test", ;
        Left = 520, ;
        Top = 370, ;
        Width = 100, ;
        Height = 30
        
    ADD OBJECT lblStatus AS Label WITH ;
        Caption = "Ready", ;
        Left = 10, ;
        Top = 410, ;
        Width = 680, ;
        Height = 20, ;
        BackColor = RGB(240, 240, 240)
    
    * Converter object
    oConverter = NULL
    
    PROCEDURE Init
        This.oConverter = CREATEOBJECT("LegacyBridge")
    ENDPROC
    
    PROCEDURE Destroy
        IF !ISNULL(This.oConverter)
            RELEASE This.oConverter
        ENDIF
    ENDPROC
    
    PROCEDURE cmdRtfToMd.Click
        LOCAL lcInput, lcOutput
        
        lcInput = ALLTRIM(ThisForm.txtInput.Value)
        ThisForm.lblStatus.Caption = "Converting RTF to Markdown..."
        
        lcOutput = ThisForm.oConverter.ConvertRtfToMarkdown(lcInput)
        
        IF !EMPTY(lcOutput)
            ThisForm.txtOutput.Value = lcOutput
            ThisForm.lblStatus.Caption = "Conversion successful!"
        ELSE
            ThisForm.txtOutput.Value = ""
            ThisForm.lblStatus.Caption = "Conversion failed: " + ;
                ThisForm.oConverter.cLastErrorMessage
        ENDIF
    ENDPROC
    
    PROCEDURE cmdMdToRtf.Click
        LOCAL lcInput, lcOutput
        
        * Switch to Markdown input if needed
        IF "{\rtf" $ ThisForm.txtInput.Value
            ThisForm.txtInput.Value = "# Hello World" + CHR(13) + CHR(10) + CHR(13) + CHR(10) + ;
                "This is a **test** document with *italic* text." + CHR(13) + CHR(10) + CHR(13) + CHR(10) + ;
                "## Features" + CHR(13) + CHR(10) + ;
                "- Bullet point 1" + CHR(13) + CHR(10) + ;
                "- Bullet point 2" + CHR(13) + CHR(10) + CHR(13) + CHR(10) + ;
                "1. Numbered item 1" + CHR(13) + CHR(10) + ;
                "2. Numbered item 2"
        ENDIF
        
        lcInput = ALLTRIM(ThisForm.txtInput.Value)
        ThisForm.lblStatus.Caption = "Converting Markdown to RTF..."
        
        lcOutput = ThisForm.oConverter.ConvertMarkdownToRtf(lcInput)
        
        IF !EMPTY(lcOutput)
            ThisForm.txtOutput.Value = lcOutput
            ThisForm.lblStatus.Caption = "Conversion successful!"
        ELSE
            ThisForm.txtOutput.Value = ""
            ThisForm.lblStatus.Caption = "Conversion failed: " + ;
                ThisForm.oConverter.cLastErrorMessage
        ENDIF
    ENDPROC
    
    PROCEDURE cmdVersion.Click
        LOCAL lcVersion
        lcVersion = ThisForm.oConverter.GetLibraryVersion()
        ThisForm.lblStatus.Caption = "LegacyBridge Version: " + lcVersion
    ENDPROC
    
    PROCEDURE cmdBatch.Click
        LOCAL i
        LOCAL ARRAY laRtfDocs[3], laMarkdownDocs[3]
        
        * Create sample RTF documents
        laRtfDocs[1] = "{\rtf1\ansi Document 1\par}"
        laRtfDocs[2] = "{\rtf1\ansi \b Document 2\b0\par}"
        laRtfDocs[3] = "{\rtf1\ansi \i Document 3\i0\par}"
        
        ThisForm.lblStatus.Caption = "Performing batch conversion..."
        
        * Perform batch conversion
        =ThisForm.oConverter.BatchConvertRtfToMarkdown(@laRtfDocs, @laMarkdownDocs)
        
        * Display results
        lcOutput = "Batch Conversion Results:" + CHR(13) + CHR(10) + CHR(13) + CHR(10)
        
        FOR i = 1 TO ALEN(laMarkdownDocs)
            lcOutput = lcOutput + "Document " + TRANSFORM(i) + ":" + CHR(13) + CHR(10)
            lcOutput = lcOutput + laMarkdownDocs[i] + CHR(13) + CHR(10) + CHR(13) + CHR(10)
        ENDFOR
        
        ThisForm.txtOutput.Value = lcOutput
        ThisForm.lblStatus.Caption = "Batch conversion completed!"
    ENDPROC
    
ENDDEFINE

* Run the form
loForm = CREATEOBJECT("TestLegacyBridgeForm")
loForm.Show()
READ EVENTS