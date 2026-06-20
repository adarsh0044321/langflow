use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use windows::Win32::System::Threading::{GetCurrentProcess, SetProcessWorkingSetSize};

// Keep track of the last time an inference request occurred
static LAST_ACTIVITY_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

pub fn update_activity() {
    let now = Instant::now();
    // Store as seconds since app start or epoch
    let elapsed = now.elapsed().as_secs();
    LAST_ACTIVITY_TIMESTAMP.store(elapsed, Ordering::SeqCst);
}

pub fn get_last_activity_elapsed() -> Duration {
    let last = LAST_ACTIVITY_TIMESTAMP.load(Ordering::SeqCst);
    let now_elapsed = Instant::now().elapsed().as_secs();
    if now_elapsed >= last {
        Duration::from_secs(now_elapsed - last)
    } else {
        Duration::from_secs(0)
    }
}

/// Native Win32 working set trimmer. Tells the OS to reclaim pageable physical memory
/// allocated to the process. This drops active RAM usage drastically.
pub fn trim_working_set() {
    unsafe {
        let process_handle = GetCurrentProcess();
        // Passing usize::MAX (-1) for both parameters forces the OS to trim the working set
        let _ = SetProcessWorkingSetSize(process_handle, usize::MAX, usize::MAX);
    }
}

/// Triggers general garbage collection and drops unused model memory.
pub fn reclaim_memory() {
    // 1. Force Rust allocator to release memory back to system
    // (on standard allocator, dropping objects frees memory, but this hints the heap manager)
    
    // 2. Call Win32 working set trimmer
    trim_working_set();
}
