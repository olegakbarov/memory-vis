use colored::*;
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

fn visualize_stack_growth(depth: usize) {
    let x = 0;
    let addr = &x as *const i32 as usize;
    println!("{:<10} {:#x}", format!("Depth {}:", depth).blue(), addr);

    if depth > 0 {
        visualize_stack_growth(depth - 1);
    }
}

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

fn visualize_memory(depth: usize) {
    let heap_usage = HEAP_USAGE.load(Ordering::SeqCst);
    let stack_usage = get_stack_usage();

    const KB: usize = 1024;
    const SCALE: usize = 64;

    let stack_blocks = (stack_usage / KB / SCALE).max(1);
    let heap_blocks = (heap_usage / KB / SCALE).max(1);

    println!("\n{}", "Memory Usage Visualization:".bold());
    println!(
        "{:<10} {} ({} KB)",
        "Stack:",
        "█".repeat(stack_blocks).blue(),
        stack_usage / KB
    );
    println!(
        "{:<10} {} ({} KB)",
        "Heap:",
        "█".repeat(heap_blocks).red(),
        heap_usage / KB
    );
    println!("\nDetailed Usage:");
    println!("{:<10} {} bytes", "Stack:".blue(), stack_usage);
    println!("{:<10} {} bytes", "Heap:".red(), heap_usage);

    println!("\n{}", "Stack Growth Visualization:".bold());
    visualize_stack_growth(depth);
}

fn main() {
    println!("{}", "Initial memory state:".bold());
    visualize_memory(3);

    // Allocate some heap memory
    let _data = vec![1, 2, 3, 4, 5];
    println!("\n{}", "After allocating small vector:".bold());
    visualize_memory(3);

    // Allocate more heap memory
    let _more_data = vec![0; 1_000_000];
    println!("\n{}", "After allocating large vector:".bold());
    visualize_memory(3);

    // Allocate a boxed value
    let _boxed = Box::new(42);
    println!("\n{}", "After allocating Box<i32>:".bold());
    visualize_memory(3);

    // Clean up
    drop(_data);
    drop(_more_data);
    drop(_boxed);
    println!("\n{}", "After deallocations:".bold());
    visualize_memory(3);
}
