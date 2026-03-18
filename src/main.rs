mod riscv;
use riscv::virtual_machine::{VirtualMachine, VMConfig};
use riscv::memory;
use std::env;
use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut config = VMConfig::new();
    let mut load_address: memory::Address128 = 0x0;
    let mut step_mode = false;
    let mut program_file: Option<String> = None;
    
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];

        if arg == "--help" || arg == "-h" {
            print_usage(&args[0]);
            return;
        } else if arg == "--memory" && i + 1 < args.len() {
            i += 1;
            config.memory_size = u64::from_str_radix(&args[i], 0).unwrap_or(memory::Memory::DEFAULT_SIZE as u64) as memory::Address128;
        } else if arg == "--debug" {
            config.debug_mode = true;
        } else if arg == "--trace" {
            config.trace_enabled = true;
        } else if arg == "--step" {
            step_mode = true;
        } else if arg == "--load-addr" && i + 1 < args.len() {
            i += 1;
            load_address = u64::from_str_radix(&args[i], 0).unwrap_or(0) as memory::Address128;
        } else if !arg.starts_with('-') {
            program_file = Some(arg.clone());
        } else {
            eprintln!("Unknown option: {}", arg);
            print_usage(&args[0]);
            return;
        }

        i += 1;
    }
    
    let mut vm = VirtualMachine::new(config);
    
    if !vm.initialize() {
        eprintln!("Failed to initialize virtual machine");
        return;
    }

    println!("RISC-V 128-bit Virtual Machine initialized");
    println!("Memory size: 0x{:x} bytes", config.memory_size);
    
    if let Some(ref file) = program_file {
        println!("Loading program: {}", file);
        println!("Load address: 0x{:x}", load_address);

        if VirtualMachine::is_assembly_file(file) {
            println!("Detected assembly file, assembling...");
            match vm.load_assembly(file, load_address) {
                Ok(_) => println!("Assembly assembled and loaded successfully"),
                Err(e) => {
                    eprintln!("Failed to assemble program: {}", e);
                    return;
                }
            }
        } else {
            match vm.load_program(file, load_address) {
                Ok(_) => println!("Program loaded successfully"),
                Err(e) => {
                    eprintln!("Failed to load program: {}", e);
                    return;
                }
            }
        }
    } else {
        println!("No program loaded. Use --help for usage information.");
        println!();
        println!("You can now use the VM programmatically:");
        println!("  - vm.load_bytes() to load code/data");
        println!("  - vm.step() to execute one instruction");
        println!("  - vm.run() to run until halt");
        println!("  - vm.print_register_state() to view registers");
        println!("  - vm.print_memory_range() to view memory");

        // 示例：打印初始状态
        println!();
        vm.print_register_state();
        return;
    }
    
    println!();
    vm.print_register_state();
    
    println!();
    if step_mode {
        println!("Running in single-step mode. Press Enter to execute next instruction, 'q' to quit.");

        vm.start();
        
        let stdin = io::stdin();
        while vm.is_running() && !vm.has_exception() {
            let mut input = String::new();
            stdin.lock().read_line(&mut input).expect("Failed to read line");

            let input = input.trim();
            if input == "q" || input == "quit" {
                println!("User requested quit");
                break;
            }

            vm.step();

            if config.trace_enabled {
                vm.print_register_state();
            }
        }
    } else {
        println!("Running program...");
        vm.run();
    }
    
    println!();
    println!("Execution finished");
    println!("Cycles: {}", vm.get_cycle_count());

    if vm.has_exception() {
        if let Some(exc) = vm.get_last_exception() {
            println!("Exception occurred: {}", exc.message);
            println!("Exception type: {:?}", exc.typ);
            println!("Exception address: 0x{:x}", exc.address);
        }
    }

    println!();
    vm.print_register_state();
}

fn print_usage(program_name: &str) {
    println!("RISC-V 128-bit Virtual Machine Framework");
    println!("Usage: {} [options] [program_file]", program_name);
    println!();
    println!("Options:");
    println!("  --help              Show this help message");
    println!("  --memory <size>     Set memory size in bytes (default: 16MB)");
    println!("  --debug             Enable debug mode");
    println!("  --trace             Enable execution tracing");
    println!("  --step              Run in single-step mode");
    println!("  --load-addr <addr>  Set program load address (default: 0x0)");
    println!();
    println!("Supported file formats:");
    println!("  .bin, .raw          Raw binary machine code");
    println!("  .s, .asm            RISC-V assembly source (auto-assembled)");
    println!();
    println!("Example:");
    println!("  {} --debug --trace program.bin", program_name);
    println!("  {} program.s", program_name);
    println!("  {} --load-addr 0x80000000 firmware.asm", program_name);
}
