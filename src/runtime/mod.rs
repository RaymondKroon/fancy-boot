extern crate llvm_sys as llvm;

use std::sync::{Once, ONCE_INIT};
use std::ffi::CString;

use llvm::prelude::*;
use llvm::core::*;
use llvm::analysis as analysis;
use llvm::execution_engine as exec;
use llvm::target as target;
use llvm::execution_engine::LLVMExecutionEngineRef;

pub trait Environment {
    fn define<T>(&mut self, name: String, val: T);
}

static INIT_LLVM: Once = ONCE_INIT;

struct LLVMEnvironment {
    context: LLVMContextRef,
    builder: LLVMBuilderRef,
    module: LLVMModuleRef,
    engine: LLVMExecutionEngineRef
}

impl LLVMEnvironment {
    pub fn new() -> Self {

        INIT_LLVM.call_once(|| {
            unsafe {
                exec::LLVMLinkInInterpreter();
                target::LLVM_InitializeNativeTarget();
            }
        });

        let context = unsafe {LLVMContextCreate()};
        let builder = unsafe {LLVMCreateBuilderInContext(context)};
        let module = unsafe {LLVMModuleCreateWithName(b"fancy\0".as_ptr() as *const _)};

        let engine = unsafe {
            let mut error = 0 as *mut i8;
            let mut exec_engine = 0 as LLVMExecutionEngineRef;
            exec::LLVMCreateExecutionEngineForModule(&mut exec_engine,
                                                     module,
                                                     &mut error);

            assert!(exec_engine != 0 as exec::LLVMExecutionEngineRef);
            exec_engine
        };

        let mut env = LLVMEnvironment{context: context, builder: builder, module: module, engine: engine};

        env.init_from_ir(String::from(
 "define i64 @\"add\"(i64, i64) {
   entry:
     %tmp = add i64 %0, %1
     ret i64 %tmp
}

define i64 @\"sub\"(i64, i64) {
   entry:
     %tmp = sub i64 %0, %1
     ret i64 %tmp
 }"));

        env
    }

    #[allow(dead_code)]
    fn llvm_dump(&mut self) {
        unsafe {
            LLVMDumpModule(self.module);
        }
    }

    fn verify(&mut self) {
        unsafe {
            let mut error = 0 as *mut i8;
            analysis::LLVMVerifyModule(self.module,
                             analysis::LLVMVerifierFailureAction::LLVMAbortProcessAction,
                             &mut error);
        }

    }

    fn init_from_ir(&mut self, ir_code: String) {
        unsafe {
            let mut msg = 0 as *mut i8;
            let len = ir_code.len();
            let cstr = cstring(ir_code);

            let membuf = LLVMCreateMemoryBufferWithMemoryRangeCopy(
                cstr,
                len as u64,
                b"fancy\0".as_ptr() as *const _);

            llvm::ir_reader::LLVMParseIRInContext(self.context,
                                                  membuf,
                                                  &mut self.module,
                                                  &mut msg);

        }
    }

    fn get_fn(&mut self, name: String) -> LLVMValueRef {
        unsafe {
            LLVMGetNamedFunction(self.module, cstring(name))
        }
    }
}

impl Drop for LLVMEnvironment {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
            LLVMDisposeModule(self.module);
            LLVMContextDispose(self.context);
        }
    }
}

fn cstring(str: String) -> *const i8 {
    CString::new(str).unwrap().as_ptr()
}

#[test]
fn demo_test() {
    demo();
}

pub fn demo() {
    println!("START");
    let mut llvm_env = LLVMEnvironment::new();
    llvm_env.verify();

    println!("VERIFIED");

    let add = llvm_env.get_fn(String::from("add"));
    let x: u64 = 2;
    let y: u64 = 3;

    println!("FUNCTION FOUND");

    let sum_result = unsafe {
        let int64 = LLVMInt64Type();

        let lx = exec::LLVMCreateGenericValueOfInt(int64, x, 1);
        let ly = exec::LLVMCreateGenericValueOfInt(int64, y, 1);

        let mut args = vec!(lx, ly);

        println!("ARGS DEFINED");

        let res = exec::LLVMRunFunction(llvm_env.engine, add, 2, args.as_mut_ptr());

        println!("EXECUTED FN");

        exec::LLVMGenericValueToInt(res, 0)
    };

    println!("LLVM SUM RESULT {}", sum_result);

    assert_eq!(5, sum_result);
}
