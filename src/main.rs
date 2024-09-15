use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct MemoryTracker;

static HEAP_USAGE: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for MemoryTracker {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let ptr = System.alloc(layout);
        HEAP_USAGE.fetch_add(size, Ordering::SeqCst);
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        System.dealloc(ptr, layout);
        HEAP_USAGE.fetch_sub(size, Ordering::SeqCst);
    }
}

#[global_allocator]
static ALLOCATOR: MemoryTracker = MemoryTracker;

fn get_stack_usage() -> usize {
    let stack_top = {
        let x = 0;
        &x as *const i32 as usize
    };

    #[inline(never)]
    fn get_approximate_stack_bottom() -> usize {
        let y = 0;
        &y as *const i32 as usize
    }

    let stack_bottom = get_approximate_stack_bottom();
    stack_top.saturating_sub(stack_bottom)
}

fn visualize_memory() {
    let heap_usage = HEAP_USAGE.load(Ordering::SeqCst);
    let stack_usage = get_stack_usage();

    println!("Memory Usage Visualization:");
    println!("Stack: {}", "█".repeat(stack_usage / 1024));
    println!("Heap:  {}", "█".repeat(heap_usage / 1024));
    println!("Stack usage: {} bytes", stack_usage);
    println!("Heap usage:  {} bytes", heap_usage);
}

fn main() {
    // Example usage
    let _data = vec![1, 2, 3, 4, 5]; // Allocate some heap memory
    visualize_memory();

    // Allocate more heap memory
    let _more_data = vec![0; 1000000];
    visualize_memory();
}
