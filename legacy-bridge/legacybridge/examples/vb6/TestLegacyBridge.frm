VERSION 5.00
Begin VB.Form frmTestLegacyBridge 
   Caption         =   "LegacyBridge Test - MD to RTF Converter"
   ClientHeight    =   7200
   ClientLeft      =   120
   ClientTop       =   465
   ClientWidth     =   10800
   LinkTopic       =   "Form1"
   ScaleHeight     =   7200
   ScaleWidth      =   10800
   StartUpPosition =   3  'Windows Default
   Begin VB.CommandButton cmdBatchConvert 
      Caption         =   "Batch Convert"
      Height          =   495
      Left            =   7560
      TabIndex        =   8
      Top             =   6600
      Width           =   1575
   End
   Begin VB.CommandButton cmdGetVersion 
      Caption         =   "Get Version"
      Height          =   495
      Left            =   9240
      TabIndex        =   7
      Top             =   6600
      Width           =   1335
   End
   Begin VB.CommandButton cmdMarkdownToRtf 
      Caption         =   "Markdown → RTF"
      Height          =   495
      Left            =   5400
      TabIndex        =   6
      Top             =   3240
      Width           =   1815
   End
   Begin VB.CommandButton cmdRtfToMarkdown 
      Caption         =   "RTF → Markdown"
      Height          =   495
      Left            =   3360
      TabIndex        =   5
      Top             =   3240
      Width           =   1815
   End
   Begin VB.TextBox txtOutput 
      Height          =   2535
      Left            =   240
      MultiLine       =   -1  'True
      ScrollBars      =   3  'Both
      TabIndex        =   3
      Top             =   4080
      Width           =   10335
   End
   Begin VB.TextBox txtInput 
      Height          =   2535
      Left            =   240
      MultiLine       =   -1  'True
      ScrollBars      =   3  'Both
      TabIndex        =   1
      Text            =   "TestLegacyBridge.frx":0000
      Top             =   480
      Width           =   10335
   End
   Begin VB.Label lblOutput 
      Caption         =   "Output:"
      Height          =   255
      Left            =   240
      TabIndex        =   4
      Top             =   3840
      Width           =   1215
   End
   Begin VB.Label lblStatus 
      BorderStyle     =   1  'Fixed Single
      Height          =   375
      Left            =   240
      TabIndex        =   2
      Top             =   6720
      Width           =   7095
   End
   Begin VB.Label lblInput 
      Caption         =   "Input:"
      Height          =   255
      Left            =   240
      TabIndex        =   0
      Top             =   240
      Width           =   1215
   End
End
Attribute VB_Name = "frmTestLegacyBridge"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Option Explicit

Private Sub Form_Load()
    ' Set default RTF content
    txtInput.Text = "{\rtf1\ansi\deff0 {\fonttbl{\f0 Times New Roman;}}" & vbCrLf & _
                    "\b Hello World\b0\par" & vbCrLf & _
                    "This is a \i test\i0 document.\par" & vbCrLf & _
                    "}"
End Sub

Private Sub cmdRtfToMarkdown_Click()
    On Error GoTo ErrorHandler
    
    Dim inputText As String
    Dim outputText As String
    
    inputText = txtInput.Text
    lblStatus.Caption = "Converting RTF to Markdown..."
    DoEvents
    
    ' Perform conversion
    outputText = ConvertRtfToMarkdown(inputText)
    
    If Len(outputText) > 0 Then
        txtOutput.Text = outputText
        lblStatus.Caption = "Conversion successful!"
    Else
        txtOutput.Text = ""
        lblStatus.Caption = "Conversion failed: " & GetLastError()
    End If
    
    Exit Sub
    
ErrorHandler:
    lblStatus.Caption = "Error: " & Err.Description
    txtOutput.Text = ""
End Sub

Private Sub cmdMarkdownToRtf_Click()
    On Error GoTo ErrorHandler
    
    Dim inputText As String
    Dim outputText As String
    
    ' Set example Markdown if input looks like RTF
    If InStr(txtInput.Text, "{\rtf") > 0 Then
        txtInput.Text = "# Hello World" & vbCrLf & vbCrLf & _
                       "This is a **test** document with *italic* text." & vbCrLf & vbCrLf & _
                       "## Features" & vbCrLf & _
                       "- Bullet point 1" & vbCrLf & _
                       "- Bullet point 2" & vbCrLf & vbCrLf & _
                       "1. Numbered item 1" & vbCrLf & _
                       "2. Numbered item 2"
    End If
    
    inputText = txtInput.Text
    lblStatus.Caption = "Converting Markdown to RTF..."
    DoEvents
    
    ' Perform conversion
    outputText = ConvertMarkdownToRtf(inputText)
    
    If Len(outputText) > 0 Then
        txtOutput.Text = outputText
        lblStatus.Caption = "Conversion successful!"
    Else
        txtOutput.Text = ""
        lblStatus.Caption = "Conversion failed: " & GetLastError()
    End If
    
    Exit Sub
    
ErrorHandler:
    lblStatus.Caption = "Error: " & Err.Description
    txtOutput.Text = ""
End Sub

Private Sub cmdGetVersion_Click()
    lblStatus.Caption = "LegacyBridge Version: " & GetLibraryVersion()
End Sub

Private Sub cmdBatchConvert_Click()
    On Error GoTo ErrorHandler
    
    Dim rtfDocs(0 To 2) As String
    Dim markdownDocs() As String
    Dim i As Integer
    
    ' Create sample RTF documents
    rtfDocs(0) = "{\rtf1\ansi Document 1\par}"
    rtfDocs(1) = "{\rtf1\ansi \b Document 2\b0\par}"
    rtfDocs(2) = "{\rtf1\ansi \i Document 3\i0\par}"
    
    lblStatus.Caption = "Performing batch conversion..."
    DoEvents
    
    ' Perform batch conversion
    markdownDocs = BatchConvertRtfToMarkdown(rtfDocs)
    
    ' Display results
    txtOutput.Text = "Batch Conversion Results:" & vbCrLf & vbCrLf
    
    For i = LBound(markdownDocs) To UBound(markdownDocs)
        txtOutput.Text = txtOutput.Text & "Document " & (i + 1) & ":" & vbCrLf
        txtOutput.Text = txtOutput.Text & markdownDocs(i) & vbCrLf & vbCrLf
    Next i
    
    lblStatus.Caption = "Batch conversion completed!"
    Exit Sub
    
ErrorHandler:
    lblStatus.Caption = "Batch conversion error: " & Err.Description
    txtOutput.Text = ""
End Sub