use colored::*;
use std::alloc::{alloc, dealloc, GlobalAlloc, Layout, System};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

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

    // The #[inline(never)] attribute ensures this function is not inlined,
    // which is crucial for accurate stack usage estimation. It forces the
    // function to have its own stack frame, creating a measurable distance
    // between this "bottom" address and the "top" address in the calling function.
    #[inline(never)]
    fn get_approximate_stack_bottom() -> usize {
        let y = 0;
        &y as *const i32 as usize
    }

    let stack_bottom = get_approximate_stack_bottom();
    stack_top.saturating_sub(stack_bottom)
}

struct MemoryState {
    allocations: Vec<(Layout, *mut u8)>,
}

impl MemoryState {
    fn new() -> Self {
        MemoryState {
            allocations: Vec::new(),
        }
    }

    fn allocate(&mut self, size: usize) {
        let layout = Layout::from_size_align(size, 8).unwrap();
        unsafe {
            let ptr = alloc(layout);
            if !ptr.is_null() {
                self.allocations.push((layout, ptr));
                // Directly update HEAP_USAGE
                HEAP_USAGE.fetch_add(layout.size(), Ordering::SeqCst);
            }
        }
    }

    fn deallocate(&mut self) {
        if let Some((layout, ptr)) = self.allocations.pop() {
            unsafe {
                dealloc(ptr, layout);
                // Directly update HEAP_USAGE
                HEAP_USAGE.fetch_sub(layout.size(), Ordering::SeqCst);
            }
        }
    }
}

impl Drop for MemoryState {
    fn drop(&mut self) {
        while !self.allocations.is_empty() {
            self.deallocate();
        }
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
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

static UPDATE_VISUALIZATION: AtomicBool = AtomicBool::new(false);

fn update_visualization() {
    UPDATE_VISUALIZATION.store(true, Ordering::SeqCst);
}

fn main() {
    let memory_state = Arc::new(Mutex::new(MemoryState::new()));
    let memory_state_clone = Arc::clone(&memory_state);

    // Spawn a thread for visualization
    thread::spawn(move || loop {
        if UPDATE_VISUALIZATION.load(Ordering::SeqCst) {
            clear_screen();
            visualize_memory(3);
            UPDATE_VISUALIZATION.store(false, Ordering::SeqCst);
        }
        thread::sleep(Duration::from_millis(100));
    });

    // Initial visualization
    update_visualization();

    // Main loop for user input
    loop {
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let parts: Vec<&str> = input.trim().split_whitespace().collect();

        match parts.get(0).map(|s| *s) {
            Some("a") => {
                if let Some(size) = parts.get(1).and_then(|s| s.parse().ok()) {
                    memory_state_clone.lock().unwrap().allocate(size);
                    update_visualization();
                } else {
                    println!("Invalid size");
                }
            }
            Some("d") => {
                memory_state_clone.lock().unwrap().deallocate();
                update_visualization();
            }
            Some("v") => {
                update_visualization();
            }
            Some("q") => break,
            _ => println!("Invalid command"),
        }
    }
}
