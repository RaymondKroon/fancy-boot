mod parser;

extern crate getopts;
extern crate llvm_sys as llvm;

use getopts::Options;
use std::env;
use std::io::stdin;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [FILE] [options]", program);
    print!("{}", opts.usage(&brief));
}

fn eval() {
    unsafe {
        // Set up a context, module and builder in that context.
        let context = llvm::core::LLVMContextCreate();
        let module = llvm::core::LLVMModuleCreateWithName(b"fancy\0".as_ptr() as *const _);
        let builder = llvm::core::LLVMCreateBuilderInContext(context);

        // Get the type signature for void nop(void);
        // Then create it in our module.
        let int64 = llvm::core::LLVMInt64Type();
        let mut int_int = vec!(int64, int64);
        let function_type = llvm::core::LLVMFunctionType(int64, int_int.as_mut_ptr(), 2, 0);
        let sum = llvm::core::LLVMAddFunction(module, b"+\0".as_ptr() as *const _,
                                                   function_type);

        // Create a basic block in the function and set our builder to generate
        // code in it.
        let bb = llvm::core::LLVMAppendBasicBlockInContext(context, sum,
                                                           b"entry\0".as_ptr() as *const _);
        llvm::core::LLVMPositionBuilderAtEnd(builder, bb);

        let tmp = llvm::core::LLVMBuildAdd(builder,
                                           llvm::core::LLVMGetParam(sum, 0),
                                           llvm::core::LLVMGetParam(sum, 1),
                                           b"tmp\0".as_ptr() as *const i8);
        llvm::core::LLVMBuildRet(builder, tmp);

        let mut error = 0 as *mut i8;
        llvm::analysis::LLVMVerifyModule(module,
                                         llvm::analysis::LLVMVerifierFailureAction::LLVMAbortProcessAction,
                                         &mut error);

        llvm::execution_engine::LLVMLinkInInterpreter();
        llvm::target::LLVM_InitializeNativeTarget();

        let mut exec_engine = 0 as llvm::execution_engine::LLVMExecutionEngineRef;
        llvm::execution_engine::LLVMCreateExecutionEngineForModule(&mut exec_engine,
                                                                   module,
                                                                   &mut error);

        assert!(exec_engine != 0 as llvm::execution_engine::LLVMExecutionEngineRef);

        let x: u64 = 2;
        let y: u64 = 3;

        let lx = llvm::execution_engine::LLVMCreateGenericValueOfInt(int64, x, 1);
        let ly = llvm::execution_engine::LLVMCreateGenericValueOfInt(int64, y, 1);

        let mut args = vec!(lx, ly);

        let res = llvm::execution_engine::LLVMRunFunction(exec_engine, sum, 2, args.as_mut_ptr());
        println!("SOM antwoord: {}", llvm::execution_engine::LLVMGenericValueToInt(res, 0));

        // Dump the module as IR to stdout.
        llvm::core::LLVMDumpModule(module);

        // Clean up. Values created in the context mostly get cleaned up there.
        llvm::core::LLVMDisposeBuilder(builder);
        llvm::core::LLVMDisposeModule(module);
        llvm::core::LLVMContextDispose(context);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("l", "llvm", "LLVM demo");
    opts.optopt("s", "", "parse expression from file", "FANCY EXPR");
    opts.optflag("p", "pipe", "parse stdin");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    if let Some(s) =  matches.opt_str("s") {
        let expressions = parser::parse_string(s);
        println!("{:?}", expressions);

        return;
    }

    if matches.opt_present("p") {
        let stdin = stdin();
        let expressions = parser::parse_buffer(stdin.lock());
        println!("{:?}", expressions);

        return;
    }

    let path = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };

    if matches.opt_present("l") {
        eval();
    }


    let expressions = parser::parse_file(path);
    println!("{:?}", expressions);

    return;
}
