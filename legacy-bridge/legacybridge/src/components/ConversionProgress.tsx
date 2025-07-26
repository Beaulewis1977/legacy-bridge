'use client'

import { motion, AnimatePresence } from 'framer-motion'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { 
  FileText, 
  CheckCircle2, 
  XCircle, 
  Clock, 
  Zap,
  Download,
  Eye,
  RotateCcw
} from 'lucide-react'
import { cn } from '@/lib/utils'
import { FileWithStatus, useFileStore } from '@/lib/stores/files'

interface ConversionProgressProps {
  className?: string
  onDownload?: (file: FileWithStatus) => void
  onPreview?: (file: FileWithStatus) => void
  onRetry?: (file: FileWithStatus) => void
}

export function ConversionProgress({ 
  className,
  onDownload,
  onPreview,
  onRetry
}: ConversionProgressProps) {
  const files = useFileStore((state) => state.files)
  const totalFiles = files.length
  const completedFiles = files.filter(f => f.status === 'completed').length
  const errorFiles = files.filter(f => f.status === 'error').length
  const convertingFiles = files.filter(f => f.status === 'converting').length
  const overallProgress = totalFiles > 0 ? (completedFiles / totalFiles) * 100 : 0

  const getStatusIcon = (status: FileWithStatus['status']) => {
    switch (status) {
      case 'completed':
        return <CheckCircle2 className="w-4 h-4 text-green-500" />
      case 'error':
        return <XCircle className="w-4 h-4 text-red-500" />
      case 'converting':
        return <Clock className="w-4 h-4 text-blue-500 animate-pulse" />
      default:
        return <FileText className="w-4 h-4 text-gray-500" />
    }
  }

  const getStatusBadgeVariant = (status: FileWithStatus['status']) => {
    switch (status) {
      case 'completed':
        return 'default' as const
      case 'error':
        return 'destructive' as const
      case 'converting':
        return 'secondary' as const
      default:
        return 'outline' as const
    }
  }

  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }

  const estimateTimeRemaining = () => {
    if (convertingFiles === 0) return null
    const avgTimePerFile = 2 // seconds (estimated)
    const remainingFiles = totalFiles - completedFiles - errorFiles
    const seconds = remainingFiles * avgTimePerFile
    if (seconds < 60) return `${seconds}s`
    return `${Math.ceil(seconds / 60)}m`
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className={cn("w-full", className)}
    >
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="text-lg font-semibold">Conversion Progress</CardTitle>
            <div className="flex items-center gap-4">
              {convertingFiles > 0 && (
                <motion.div
                  initial={{ opacity: 0, scale: 0.8 }}
                  animate={{ opacity: 1, scale: 1 }}
                  className="flex items-center gap-2 text-sm text-muted-foreground"
                >
                  <Zap className="w-4 h-4 text-yellow-500 animate-pulse" />
                  <span>ETA: {estimateTimeRemaining()}</span>
                </motion.div>
              )}
              <Badge variant="outline">
                {completedFiles}/{totalFiles} files
              </Badge>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {/* Overall Progress */}
          <div className="mb-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium">Overall Progress</span>
              <span className="text-sm text-muted-foreground">{Math.round(overallProgress)}%</span>
            </div>
            <Progress value={overallProgress} className="h-2" />
          </div>

          {/* File List */}
          <div className="space-y-3">
            <AnimatePresence mode="popLayout">
              {files.map((file) => (
                <motion.div
                  key={file.id}
                  layout
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: 20 }}
                  className="border rounded-lg p-3 bg-background"
                >
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-2 flex-1 min-w-0">
                      {getStatusIcon(file.status)}
                      <span className="text-sm font-medium truncate">{file.name || 'Unknown file'}</span>
                      <Badge variant={getStatusBadgeVariant(file.status)} className="text-xs">
                        {file.status}
                      </Badge>
                    </div>
                    <div className="flex items-center gap-2">
                      {file.status === 'completed' && (
                        <>
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={() => onPreview?.(file)}
                            className="h-7 px-2"
                          >
                            <Eye className="w-3 h-3 mr-1" />
                            Preview
                          </Button>
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={() => onDownload?.(file)}
                            className="h-7 px-2"
                          >
                            <Download className="w-3 h-3 mr-1" />
                            Download
                          </Button>
                        </>
                      )}
                      {file.status === 'error' && (
                        <Button
                          size="sm"
                          variant="ghost"
                          onClick={() => onRetry?.(file)}
                          className="h-7 px-2"
                        >
                          <RotateCcw className="w-3 h-3 mr-1" />
                          Retry
                        </Button>
                      )}
                    </div>
                  </div>

                  {/* Progress Bar for Converting Files */}
                  {file.status === 'converting' && (
                    <motion.div
                      initial={{ opacity: 0, height: 0 }}
                      animate={{ opacity: 1, height: 'auto' }}
                      className="mt-2"
                    >
                      <Progress value={file.progress} className="h-1" />
                    </motion.div>
                  )}

                  {/* File Details */}
                  <div className="flex items-center gap-4 mt-2 text-xs text-muted-foreground">
                    <span>{formatFileSize(file.size || 0)}</span>
                    {file.conversionTime && (
                      <span>Converted in {file.conversionTime.toFixed(1)}s</span>
                    )}
                    {file.error && (
                      <span className="text-red-500 truncate flex-1">
                        Error: {file.error}
                      </span>
                    )}
                  </div>
                </motion.div>
              ))}
            </AnimatePresence>
          </div>

          {/* Summary Stats */}
          {totalFiles > 0 && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.3 }}
              className="mt-6 pt-4 border-t"
            >
              <div className="grid grid-cols-3 gap-4 text-center">
                <div>
                  <div className="text-2xl font-bold text-green-500">{completedFiles}</div>
                  <div className="text-xs text-muted-foreground">Completed</div>
                </div>
                <div>
                  <div className="text-2xl font-bold text-blue-500">{convertingFiles}</div>
                  <div className="text-xs text-muted-foreground">Processing</div>
                </div>
                <div>
                  <div className="text-2xl font-bold text-red-500">{errorFiles}</div>
                  <div className="text-xs text-muted-foreground">Failed</div>
                </div>
              </div>
            </motion.div>
          )}
        </CardContent>
      </Card>
    </motion.div>
  )
}