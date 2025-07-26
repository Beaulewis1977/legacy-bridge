# Frontend Memory Leak Fixes

## Identified Memory Leaks

### 1. Progress Update Intervals (CRITICAL)
**Location**: `/root/repo/legacybridge/src/app/page.tsx`
- **Issue**: `setInterval` for progress updates not properly cleaned up
- **Impact**: Memory leak grows with each conversion, intervals keep running
- **Fix**: Track intervals in ref and ensure cleanup

### 2. Download Manager Polling
**Location**: `/root/repo/legacybridge/src/components/DownloadManager.tsx`
- **Issue**: Polling interval not cleaned up on unmount
- **Fix**: Add cleanup in useEffect return

### 3. Cache Timeout Cleanup
**Locations**: Multiple auth/tenancy components
- **Issue**: `setTimeout` callbacks creating memory leaks
- **Fix**: Store timeout IDs and clear on cleanup

## Fixes Applied

### 1. Progress Interval Fix
```typescript
// Track intervals to prevent memory leaks
const progressIntervalsRef = useRef<Map<string, NodeJS.Timeout>>(new Map());

// Clear any existing intervals
progressIntervalsRef.current.forEach(interval => clearInterval(interval));
progressIntervalsRef.current.clear();

// Store interval reference
progressIntervalsRef.current.set(file.id, progressInterval);

// Cleanup on unmount
useEffect(() => {
  return () => {
    progressIntervalsRef.current.forEach(interval => clearInterval(interval));
    progressIntervalsRef.current.clear();
  };
}, []);
```

### 2. Download Manager Fix
```typescript
useEffect(() => {
  const interval = setInterval(() => {
    setActiveDownloads(downloadService.getActiveDownloads());
  }, 100);
  
  // Cleanup on unmount
  return () => clearInterval(interval);
}, []);
```

### 3. Cache Timeout Fix
```typescript
class RBACService {
  private timeoutRefs = new Map<string, NodeJS.Timeout>();
  
  cacheUser(userId: string, user: User) {
    // Clear existing timeout if any
    const existingTimeout = this.timeoutRefs.get(userId);
    if (existingTimeout) clearTimeout(existingTimeout);
    
    // Set new timeout
    const timeout = setTimeout(() => {
      this.userCache.delete(userId);
      this.timeoutRefs.delete(userId);
    }, 5 * 60 * 1000);
    
    this.timeoutRefs.set(userId, timeout);
  }
  
  cleanup() {
    this.timeoutRefs.forEach(timeout => clearTimeout(timeout));
    this.timeoutRefs.clear();
  }
}
```

## Additional Recommendations

1. **Use AbortController** for async operations
2. **Implement proper cleanup** in all useEffect hooks
3. **Use WeakMap** for caches where appropriate
4. **Add memory monitoring** to detect leaks early
5. **Implement periodic cache cleanup** instead of individual timeouts