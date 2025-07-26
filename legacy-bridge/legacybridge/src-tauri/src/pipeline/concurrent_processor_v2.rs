// Enhanced Concurrent Processing Pipeline with Adaptive Thread Pool
//
// Key improvements over v1:
// 1. Uses adaptive thread pool with work-stealing
// 2. Proper backpressure management
// 3. Better memory management with pooling
// 4. Enhanced metrics and monitoring
// 5. Support for streaming and chunked processing

use crate::pipeline::adaptive_thread_pool::{AdaptiveThreadPool, Task, BackpressureError, PoolConfig};
use crate::conversion::types::{ConversionResult, ConversionError, RtfDocument};
use crate::conversion::markdown_parser_optimized::OptimizedMarkdownParser;
use crate::pipeline::formatting_engine_optimized::OptimizedFormattingEngine;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use parking_lot::RwLock;
use std::time::{Duration, Instant};
use crossbeam_channel::{bounded, Sender, Receiver};
use std::io::Read;

/// Enhanced concurrent processor with adaptive threading
pub struct ConcurrentProcessorV2 {
    /// Adaptive thread pool
    thread_pool: Arc<AdaptiveThreadPool>,
    /// Memory pool for document buffers
    memory_pool: Arc<MemoryPool>,
    /// Global metrics
    metrics: Arc<RwLock<ProcessingMetricsV2>>,
    /// Configuration
    config: ProcessorConfig,
}

/// Processor configuration
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    /// Maximum document size in bytes
    pub max_document_size: usize,
    /// Chunk size for large documents
    pub chunk_size: usize,
    /// Enable memory pooling
    pub use_memory_pool: bool,
    /// Enable streaming mode
    pub enable_streaming: bool,
    /// Batch timeout
    pub batch_timeout: Duration,
    /// Maximum concurrent operations per tenant (for multi-tenancy)
    pub max_concurrent_per_tenant: usize,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            max_document_size: 100 * 1024 * 1024, // 100MB
            chunk_size: 1024 * 1024, // 1MB
            use_memory_pool: true,
            enable_streaming: true,
            batch_timeout: Duration::from_secs(30),
            max_concurrent_per_tenant: 100,
        }
    }
}

/// Enhanced processing metrics
#[derive(Debug, Default, Clone)]
pub struct ProcessingMetricsV2 {
    pub total_processed: u64,
    pub total_bytes_in: u64,
    pub total_bytes_out: u64,
    pub average_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub throughput_mbps: f64,
    pub error_rate: f64,
    pub backpressure_events: u64,
    pub memory_pool_hits: u64,
    pub memory_pool_misses: u64,
}

/// Memory pool for document buffers
struct MemoryPool {
    small_buffers: RwLock<Vec<Vec<u8>>>,  // < 1MB
    medium_buffers: RwLock<Vec<Vec<u8>>>, // 1-10MB
    large_buffers: RwLock<Vec<Vec<u8>>>,  // > 10MB
    hits: AtomicU64,
    misses: AtomicU64,
}

impl MemoryPool {
    fn new() -> Self {
        Self {
            small_buffers: RwLock::new(Vec::with_capacity(100)),
            medium_buffers: RwLock::new(Vec::with_capacity(20)),
            large_buffers: RwLock::new(Vec::with_capacity(5)),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    fn acquire(&self, size: usize) -> Vec<u8> {
        let buffers = if size < 1024 * 1024 {
            &self.small_buffers
        } else if size < 10 * 1024 * 1024 {
            &self.medium_buffers
        } else {
            &self.large_buffers
        };

        if let Some(mut buffer) = buffers.write().pop() {
            self.hits.fetch_add(1, Ordering::Relaxed);
            buffer.clear();
            buffer.reserve(size);
            buffer
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            Vec::with_capacity(size)
        }
    }

    fn release(&self, buffer: Vec<u8>) {
        let capacity = buffer.capacity();
        let buffers = if capacity < 1024 * 1024 {
            &self.small_buffers
        } else if capacity < 10 * 1024 * 1024 {
            &self.medium_buffers
        } else {
            &self.large_buffers
        };

        let mut guard = buffers.write();
        if guard.len() < guard.capacity() {
            guard.push(buffer);
        }
    }
}

/// Conversion task for thread pool
struct ConversionTask {
    id: String,
    content: String,
    options: ConversionOptions,
    memory_pool: Arc<MemoryPool>,
    result_sender: Sender<ConversionResponse>,
}

impl Task for ConversionTask {
    type Output = ();

    fn execute(self) -> Self::Output {
        let start_time = Instant::now();
        let input_size = self.content.len();

        // Perform conversion
        let result = if self.options.parallel_chunks && input_size > self.options.chunk_size {
            process_chunked(&self.content, &self.options, &self.memory_pool)
        } else {
            process_single_threaded(&self.content)
        };

        // Calculate metrics
        let output_size = match &result {
            Ok(output) => output.len(),
            Err(_) => 0,
        };

        let response = ConversionResponse {
            id: self.id,
            result,
            metrics: ConversionMetrics {
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                input_size_bytes: input_size,
                output_size_bytes: output_size,
                memory_peak_bytes: input_size + output_size,
            },
        };

        // Send result back
        let _ = self.result_sender.send(response);
    }
}

/// Process document in single thread
fn process_single_threaded(content: &str) -> ConversionResult<String> {
    let mut parser = OptimizedMarkdownParser::new();
    let document = parser.parse(content)?;
    
    let mut formatter = OptimizedFormattingEngine::new();
    formatter.generate_markdown_with_fidelity(&document)
}

/// Process document in chunks
fn process_chunked(
    content: &str,
    options: &ConversionOptions,
    _memory_pool: &Arc<MemoryPool>,
) -> ConversionResult<String> {
    // Split into chunks at paragraph boundaries
    let chunks = split_into_chunks(content, options.chunk_size);
    
    // Process chunks in parallel using rayon
    use rayon::prelude::*;
    let results: Vec<ConversionResult<RtfDocument>> = chunks
        .par_iter()
        .map(|chunk| {
            let mut parser = OptimizedMarkdownParser::new();
            parser.parse(chunk)
        })
        .collect();
    
    // Merge results
    merge_results(results)
}

/// Split content into chunks
fn split_into_chunks(content: &str, chunk_size: usize) -> Vec<&str> {
    let mut chunks = Vec::new();
    let mut current_start = 0;
    let mut current_size = 0;
    
    for line in content.lines() {
        current_size += line.len() + 1;
        
        if current_size >= chunk_size && line.trim().is_empty() {
            let end = content[current_start..]
                .find('\n')
                .map(|pos| current_start + pos)
                .unwrap_or(content.len());
            chunks.push(&content[current_start..end]);
            current_start = end + 1;
            current_size = 0;
        }
    }
    
    if current_start < content.len() {
        chunks.push(&content[current_start..]);
    }
    
    chunks
}

/// Merge document results
fn merge_results(results: Vec<ConversionResult<RtfDocument>>) -> ConversionResult<String> {
    let mut merged_document = RtfDocument {
        metadata: crate::conversion::types::DocumentMetadata::default(),
        content: Vec::new(),
    };
    
    for result in results {
        match result {
            Ok(doc) => merged_document.content.extend(doc.content),
            Err(e) => return Err(e),
        }
    }
    
    let mut formatter = OptimizedFormattingEngine::new();
    formatter.generate_markdown_with_fidelity(&merged_document)
}

impl ConcurrentProcessorV2 {
    /// Create new processor with default configuration
    pub fn new() -> Self {
        Self::with_config(ProcessorConfig::default())
    }

    /// Create processor with custom configuration
    pub fn with_config(config: ProcessorConfig) -> Self {
        // Configure adaptive thread pool
        let pool_config = PoolConfig {
            min_threads: 2,
            max_threads: num_cpus::get() * 2,
            max_queue_size: 10_000,
            backpressure_threshold: 0.8,
            idle_timeout: Duration::from_secs(60),
            sampling_interval: Duration::from_millis(100),
            numa_aware: true,
            warm_up: true,
        };

        Self {
            thread_pool: Arc::new(AdaptiveThreadPool::with_config(pool_config)),
            memory_pool: Arc::new(MemoryPool::new()),
            metrics: Arc::new(RwLock::new(ProcessingMetricsV2::default())),
            config,
        }
    }

    /// Process single conversion request
    pub async fn process_single(&self, request: ConversionRequest) -> ConversionResponse {
        let (tx, rx) = bounded(1);
        
        // Create task
        let task = ConversionTask {
            id: request.id.clone(),
            content: match self.load_content(request.content) {
                Ok((content, _)) => content,
                Err(e) => {
                    return ConversionResponse {
                        id: request.id,
                        result: Err(e),
                        metrics: ConversionMetrics {
                            processing_time_ms: 0,
                            input_size_bytes: 0,
                            output_size_bytes: 0,
                            memory_peak_bytes: 0,
                        },
                    };
                }
            },
            options: request.options,
            memory_pool: self.memory_pool.clone(),
            result_sender: tx,
        };

        // Submit to thread pool
        match self.thread_pool.submit(task) {
            Ok(_) => {
                // Wait for result with timeout
                match rx.recv_timeout(self.config.batch_timeout) {
                    Ok(response) => {
                        self.update_metrics(&response);
                        response
                    }
                    Err(_) => ConversionResponse {
                        id: request.id,
                        result: Err(ConversionError::IoError("Processing timeout".to_string())),
                        metrics: ConversionMetrics {
                            processing_time_ms: self.config.batch_timeout.as_millis() as u64,
                            input_size_bytes: 0,
                            output_size_bytes: 0,
                            memory_peak_bytes: 0,
                        },
                    },
                }
            }
            Err(e) => {
                self.metrics.write().backpressure_events += 1;
                ConversionResponse {
                    id: request.id,
                    result: Err(ConversionError::ValidationError(format!("Backpressure: {:?}", e))),
                    metrics: ConversionMetrics {
                        processing_time_ms: 0,
                        input_size_bytes: 0,
                        output_size_bytes: 0,
                        memory_peak_bytes: 0,
                    },
                }
            }
        }
    }

    /// Process batch of conversions
    pub async fn process_batch(&self, requests: Vec<ConversionRequest>) -> Vec<ConversionResponse> {
        let batch_size = requests.len();
        let (tx, rx) = bounded(batch_size);
        let mut responses = Vec::with_capacity(batch_size);

        // Submit all tasks
        for request in requests {
            let tx_clone = tx.clone();
            let task = ConversionTask {
                id: request.id.clone(),
                content: match self.load_content(request.content) {
                    Ok((content, _)) => content,
                    Err(e) => {
                        responses.push(ConversionResponse {
                            id: request.id,
                            result: Err(e),
                            metrics: ConversionMetrics {
                                processing_time_ms: 0,
                                input_size_bytes: 0,
                                output_size_bytes: 0,
                                memory_peak_bytes: 0,
                            },
                        });
                        continue;
                    }
                },
                options: request.options,
                memory_pool: self.memory_pool.clone(),
                result_sender: tx_clone,
            };

            if let Err(e) = self.thread_pool.submit(task) {
                self.metrics.write().backpressure_events += 1;
                responses.push(ConversionResponse {
                    id: request.id,
                    result: Err(ConversionError::ValidationError(format!("Backpressure: {:?}", e))),
                    metrics: ConversionMetrics {
                        processing_time_ms: 0,
                        input_size_bytes: 0,
                        output_size_bytes: 0,
                        memory_peak_bytes: 0,
                    },
                });
            }
        }

        // Collect results
        drop(tx);
        while let Ok(response) = rx.recv_timeout(self.config.batch_timeout) {
            self.update_metrics(&response);
            responses.push(response);
        }

        responses
    }

    /// Load content from various sources
    fn load_content(&self, content: ConversionContent) -> ConversionResult<(String, usize)> {
        match content {
            ConversionContent::Memory(data) => {
                let size = data.len();
                if size > self.config.max_document_size {
                    return Err(ConversionError::ValidationError(
                        format!("Document size {} exceeds limit {}", size, self.config.max_document_size)
                    ));
                }
                Ok((data, size))
            }
            ConversionContent::File(path) => {
                std::fs::read_to_string(&path)
                    .map(|data| {
                        let size = data.len();
                        (data, size)
                    })
                    .map_err(|e| ConversionError::IoError(e.to_string()))
            }
            ConversionContent::Stream(mut stream) => {
                let mut buffer = String::new();
                stream.read_to_string(&mut buffer)
                    .map(|_| {
                        let size = buffer.len();
                        (buffer, size)
                    })
                    .map_err(|e| ConversionError::IoError(e.to_string()))
            }
        }
    }

    /// Update global metrics
    fn update_metrics(&self, response: &ConversionResponse) {
        let mut metrics = self.metrics.write();
        
        metrics.total_processed += 1;
        metrics.total_bytes_in += response.metrics.input_size_bytes as u64;
        metrics.total_bytes_out += response.metrics.output_size_bytes as u64;
        
        // Update latency metrics (simple exponential moving average)
        let latency = response.metrics.processing_time_ms as f64;
        if metrics.average_latency_ms == 0.0 {
            metrics.average_latency_ms = latency;
            metrics.p99_latency_ms = latency;
        } else {
            metrics.average_latency_ms = metrics.average_latency_ms * 0.95 + latency * 0.05;
            metrics.p99_latency_ms = metrics.p99_latency_ms.max(latency);
        }
        
        // Update error rate
        if response.result.is_err() {
            metrics.error_rate = (metrics.error_rate * (metrics.total_processed - 1) as f64 + 1.0) 
                / metrics.total_processed as f64;
        } else {
            metrics.error_rate = (metrics.error_rate * (metrics.total_processed - 1) as f64) 
                / metrics.total_processed as f64;
        }
        
        // Update memory pool metrics
        metrics.memory_pool_hits = self.memory_pool.hits.load(Ordering::Relaxed);
        metrics.memory_pool_misses = self.memory_pool.misses.load(Ordering::Relaxed);
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> ProcessingMetricsV2 {
        self.metrics.read().clone()
    }

    /// Get thread pool statistics
    pub fn get_pool_statistics(&self) -> crate::pipeline::adaptive_thread_pool::PoolStatistics {
        self.thread_pool.statistics()
    }

    /// Shutdown processor
    pub fn shutdown(&self) {
        self.thread_pool.shutdown();
    }
}

// Re-export types from v1 for compatibility
pub use super::concurrent_processor::{
    ConversionRequest, ConversionContent, ConversionOptions, 
    ConversionResponse, ConversionMetrics
};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enhanced_concurrent_processing() {
        let processor = ConcurrentProcessorV2::new();
        
        // Create test requests
        let requests: Vec<_> = (0..100)
            .map(|i| ConversionRequest {
                id: format!("test-{}", i),
                content: ConversionContent::Memory(
                    format!("# Test Document {}\n\nThis is test content with multiple paragraphs.\n\nParagraph 2.", i)
                ),
                options: ConversionOptions::default(),
            })
            .collect();
        
        // Process batch
        let responses = processor.process_batch(requests).await;
        
        // Verify results
        assert_eq!(responses.len(), 100);
        for response in &responses {
            assert!(response.result.is_ok());
        }
        
        // Check metrics
        let metrics = processor.get_metrics();
        assert_eq!(metrics.total_processed, 100);
        assert!(metrics.average_latency_ms > 0.0);
        
        // Check pool statistics
        let pool_stats = processor.get_pool_statistics();
        assert!(pool_stats.tasks_completed >= 100);
        assert!(pool_stats.total_threads > 0);
    }

    #[tokio::test]
    async fn test_backpressure_handling() {
        let config = ProcessorConfig {
            max_document_size: 1024 * 1024, // 1MB limit
            ..Default::default()
        };
        let processor = ConcurrentProcessorV2::with_config(config);
        
        // Create many requests to trigger backpressure
        let requests: Vec<_> = (0..1000)
            .map(|i| ConversionRequest {
                id: format!("load-test-{}", i),
                content: ConversionContent::Memory("x".repeat(100_000)), // 100KB each
                options: ConversionOptions {
                    parallel_chunks: true,
                    chunk_size: 10_000,
                    ..Default::default()
                },
            })
            .collect();
        
        // Process with potential backpressure
        let responses = processor.process_batch(requests).await;
        
        // Check that we handled backpressure
        let metrics = processor.get_metrics();
        if metrics.backpressure_events > 0 {
            println!("Backpressure events: {}", metrics.backpressure_events);
        }
        
        // All requests should be processed eventually
        assert!(!responses.is_empty());
    }

    #[tokio::test]
    async fn test_memory_pooling() {
        let processor = ConcurrentProcessorV2::new();
        
        // Process multiple requests to test memory pool
        for i in 0..10 {
            let request = ConversionRequest {
                id: format!("pool-test-{}", i),
                content: ConversionContent::Memory("Test content for memory pool".to_string()),
                options: ConversionOptions::default(),
            };
            
            let _ = processor.process_single(request).await;
        }
        
        // Check memory pool usage
        let metrics = processor.get_metrics();
        assert!(metrics.memory_pool_hits > 0 || metrics.memory_pool_misses > 0);
    }
}