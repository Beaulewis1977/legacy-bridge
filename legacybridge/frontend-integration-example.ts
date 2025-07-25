// Example TypeScript code showing how to use the new Tauri commands from the frontend

import { invoke } from '@tauri-apps/api/tauri';

// Response types matching the Rust structures
interface ConversionResponse {
  success: boolean;
  result?: string;
  error?: string;
}

interface FileOperationResponse {
  success: boolean;
  path?: string;
  content?: string;
  error?: string;
}

interface BatchConversionRequest {
  input_paths: string[];
  output_directory: string;
}

interface BatchConversionResponse {
  success: boolean;
  converted_files: string[];
  failed_files: [string, string][];
  error?: string;
}

// Example usage of the new commands

// 1. Read an RTF file and convert it to Markdown
async function readAndConvertRTF(filePath: string): Promise<string | null> {
  try {
    const response = await invoke<FileOperationResponse>('read_rtf_file', {
      file_path: filePath
    });
    
    if (response.success && response.content) {
      console.log(`Successfully converted ${filePath}`);
      return response.content;
    } else {
      console.error(`Failed to convert ${filePath}: ${response.error}`);
      return null;
    }
  } catch (error) {
    console.error('Error calling read_rtf_file:', error);
    return null;
  }
}

// 2. Write Markdown content to a file
async function saveMarkdownFile(filePath: string, content: string): Promise<boolean> {
  try {
    const response = await invoke<FileOperationResponse>('write_markdown_file', {
      file_path: filePath,
      content: content
    });
    
    if (response.success) {
      console.log(`Successfully saved to ${filePath}`);
      return true;
    } else {
      console.error(`Failed to save ${filePath}: ${response.error}`);
      return false;
    }
  } catch (error) {
    console.error('Error calling write_markdown_file:', error);
    return false;
  }
}

// 3. Read file as base64 (useful for binary files)
async function readFileAsBase64(filePath: string): Promise<string | null> {
  try {
    const response = await invoke<FileOperationResponse>('read_file_base64', {
      file_path: filePath
    });
    
    if (response.success && response.content) {
      return response.content;
    } else {
      console.error(`Failed to read ${filePath}: ${response.error}`);
      return null;
    }
  } catch (error) {
    console.error('Error calling read_file_base64:', error);
    return null;
  }
}

// 4. Write base64 content to file
async function writeFileFromBase64(filePath: string, base64Content: string): Promise<boolean> {
  try {
    const response = await invoke<FileOperationResponse>('write_file_base64', {
      file_path: filePath,
      base64_content: base64Content
    });
    
    if (response.success) {
      console.log(`Successfully wrote base64 to ${filePath}`);
      return true;
    } else {
      console.error(`Failed to write ${filePath}: ${response.error}`);
      return false;
    }
  } catch (error) {
    console.error('Error calling write_file_base64:', error);
    return false;
  }
}

// 5. Batch convert multiple RTF files
async function batchConvertRTFFiles(inputPaths: string[], outputDir: string): Promise<void> {
  try {
    const request: BatchConversionRequest = {
      input_paths: inputPaths,
      output_directory: outputDir
    };
    
    const response = await invoke<BatchConversionResponse>('batch_convert_rtf_to_markdown', {
      request: request
    });
    
    if (response.success) {
      console.log('Batch conversion completed successfully!');
      console.log('Converted files:', response.converted_files);
    } else {
      console.error('Batch conversion had errors:');
      response.failed_files.forEach(([path, error]) => {
        console.error(`  ${path}: ${error}`);
      });
    }
  } catch (error) {
    console.error('Error calling batch_convert_rtf_to_markdown:', error);
  }
}

// Example usage in a React component
export function useFileConversion() {
  const handleSingleFileConversion = async (rtfPath: string, outputPath: string) => {
    const markdown = await readAndConvertRTF(rtfPath);
    if (markdown) {
      await saveMarkdownFile(outputPath, markdown);
    }
  };

  const handleBatchConversion = async (files: string[], outputDir: string) => {
    await batchConvertRTFFiles(files, outputDir);
  };

  return {
    handleSingleFileConversion,
    handleBatchConversion,
    readAndConvertRTF,
    saveMarkdownFile,
    readFileAsBase64,
    writeFileFromBase64
  };
}