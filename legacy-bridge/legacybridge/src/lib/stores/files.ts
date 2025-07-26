import { create } from 'zustand';
import { FileInfo, ConversionResult } from '../tauri-api';

export interface FileWithStatus extends FileInfo {
  id: string;
  status: 'idle' | 'converting' | 'completed' | 'error';
  result?: ConversionResult;
  progress?: number;
}

interface FileStore {
  files: FileWithStatus[];
  addFiles: (files: File[]) => void;
  removeFile: (id: string) => void;
  updateFileStatus: (id: string, status: FileWithStatus['status'], result?: ConversionResult) => void;
  updateFileProgress: (id: string, progress: number) => void;
  clearFiles: () => void;
  getFileById: (id: string) => FileWithStatus | undefined;
}

export const useFileStore = create<FileStore>((set, get) => ({
  files: [],
  
  addFiles: (newFiles: File[]) => {
    const filesWithStatus: FileWithStatus[] = newFiles
      .filter(file => file.name.endsWith('.rtf') || file.name.endsWith('.md'))
      .map(file => ({
        id: `${file.name}-${Date.now()}-${Math.random()}`,
        name: file.name,
        path: file.webkitRelativePath || file.name,
        size: file.size,
        type: file.name.endsWith('.rtf') ? 'rtf' : 'md',
        status: 'idle' as const
      }));
    
    set((state) => ({
      files: [...state.files, ...filesWithStatus]
    }));
  },
  
  removeFile: (id: string) => {
    set((state) => ({
      files: state.files.filter(file => file.id !== id)
    }));
  },
  
  updateFileStatus: (id: string, status: FileWithStatus['status'], result?: ConversionResult) => {
    set((state) => ({
      files: state.files.map(file =>
        file.id === id
          ? { ...file, status, result, progress: status === 'completed' ? 100 : file.progress }
          : file
      )
    }));
  },
  
  updateFileProgress: (id: string, progress: number) => {
    set((state) => ({
      files: state.files.map(file =>
        file.id === id ? { ...file, progress } : file
      )
    }));
  },
  
  clearFiles: () => {
    set({ files: [] });
  },
  
  getFileById: (id: string) => {
    return get().files.find(file => file.id === id);
  }
}));