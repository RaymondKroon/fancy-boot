extern crate llvm_sys as llvm;
extern crate libc;

use std::collections::HashMap;
use std::sync::{Once, ONCE_INIT};
use std::ffi::{CString, CStr};
use std::str;

use llvm::prelude::*;
use llvm::core::*;
use llvm::analysis as analysis;
use llvm::execution_engine as exec;
use llvm::target as target;
use llvm::execution_engine::LLVMExecutionEngineRef;

use ::parser::Expression;

pub trait Value {
    fn dump(&mut self) -> Self;
}

pub trait Environment<V: Value> {
    fn eval_all(&mut self, expressions: Vec<Expression>) -> V;
    fn eval(&mut self, expression: Expression) -> V;
}

static INIT_LLVM: Once = ONCE_INIT;

pub struct LLVMEnvironment {
    context: LLVMContextRef,
    builder: LLVMBuilderRef,
    module: LLVMModuleRef,
    engine: LLVMExecutionEngineRef,
    named_values: HashMap<String, LLVMValueRef>
}

impl Value for LLVMValueRef {
    fn dump(&mut self) -> Self {
        unsafe {LLVMDumpValue(*self);}

        *self
    }
}

impl Environment<LLVMValueRef> for LLVMEnvironment {
    fn eval_all(&mut self, expressions: Vec<Expression>) -> LLVMValueRef {
        let mut result = 0 as LLVMValueRef;
        for e in expressions {
            result = self.eval(e);
        }

        result
    }

    fn eval(&mut self, expression: Expression) -> LLVMValueRef {
        let result = match expression {
            Expression::Symbol(symbol) => self.eval_symbol(symbol),
            Expression::Number(number) => self.eval_number(number),
            Expression::String(s) => self.eval_string(s),
            Expression::SExpression(sexp) => self.eval_sexp(sexp),
            Expression::Params(params) => unreachable!("Params")
        };

        result
    }
}

impl LLVMEnvironment {

    fn eval_symbol(&mut self, symbol: String) -> LLVMValueRef {
        if self.named_values.contains_key(&symbol) {
            *self.named_values.get(&symbol).unwrap()
        }
        else {
            panic!("Undefined symbol {}", symbol);
        }
    }

    fn eval_number(&mut self, number: String) -> LLVMValueRef {
        let result = self.eval(Expression::SExpression(vec!(
            ::parser::symbol("atof"), Expression::String(number))));

        asserted_return(result)
    }

    fn eval_string(&mut self, s: String) -> LLVMValueRef {
        unsafe {
            let len = s.len();
            let result = LLVMConstString(cstring(s), len as u32, 0);

            asserted_return(result)
        }
    }

    fn eval_sexp(&mut self, sexp: Vec<Expression>) -> LLVMValueRef {
        if sexp.len() > 0 {
            if let Expression::Symbol(name) = sexp[0].clone() {

                match name.as_ref() {
                    "extern" => self.defextern(sexp[1..].to_vec()).dump(),
                    "fn" => self.defn(sexp[1..].to_vec()).dump(),
                    _ => self.eval_fn(name, sexp[1..].to_vec()).dump()
                }
            }
            else {
                panic!("Expected symbol, got {:?}", sexp[0]);
            }
        }
        else {
            // should be unit, 0 for now
            self.eval(Expression::Number(String::from("0")))
        }
    }

    fn eval_fn(&mut self, name: String, args: Vec<Expression>) -> LLVMValueRef {

        let function = self.get_fn(&name);

        if function == 0 as LLVMValueRef {
            panic!("Undefined function {}", name);
        }

        let arg_count = unsafe{LLVMCountParams(function)};

        if arg_count != args.len() as u32 {
            panic!("{} requires {} args, got {}", name, arg_count, args.len());
        }

        let mut fn_args = Vec::<LLVMValueRef>::new();
        for a in args {
            fn_args.push(self.eval(a));
        }

        unsafe {
            LLVMBuildCall(self.builder, function, fn_args.as_mut_ptr(), arg_count, cstring_a("tmpcall"))
        }
    }

    fn defextern(&mut self, args: Vec<Expression>) -> LLVMValueRef {
        if args.len() == 2 {
            if let Expression::Symbol(name) = args[0].clone() {
                let mut param_types = Vec::<LLVMTypeRef>::with_capacity(args.len() - 1);

                if let Expression::Params(params) = args[1].clone() {

                    unsafe {

                        for a in args[1..].to_vec() {
                            param_types.push(LLVMDoubleType());
                        }


                        let function_type = LLVMFunctionType(LLVMDoubleType(),
                                                             param_types.as_mut_ptr(),
                                                             param_types.len() as u32, 0);

                        let function = LLVMAddFunction(self.module, cstring(name), function_type);


                        let llvm_params = LLVMEnvironment::get_params(function);

                        for i in 0..params.len() {
                            if let Expression::Symbol(name) = params[i].clone() {
                                LLVMSetValueName(llvm_params[i],  cstring(name));
                            }
                        }

                        function
                    }
                }
                else {
                    panic!("Expected params, got {:?}", args[1]);
                }
            }
            else {
                panic!("Expected symbol, got {:?}", args[0]);
            }
        }
        else {
            panic!("extern requires 2 params");
        }
    }

    fn defn(&mut self, args: Vec<Expression>) -> LLVMValueRef {
        if args.len() >= 2 {
            let fndef = self.defextern(args[0..2].to_vec());

            unsafe {
                let bb = LLVMAppendBasicBlockInContext(self.context, fndef, cstring_a("entry"));
	        LLVMPositionBuilderAtEnd(self.builder, bb);

                self.named_values.clear();
                for param in LLVMEnvironment::get_params(fndef) {
                    self.named_values.insert(buf_to_string(LLVMGetValueName(param)), param);
                }

                let inner = self.eval_all(args[2..].to_vec());

	        LLVMBuildRet(self.builder, inner);

                analysis::LLVMVerifyFunction(fndef,
                                               analysis::LLVMVerifierFailureAction::LLVMAbortProcessAction);

                fndef
            }
        }
        else {
            panic!("fn requires at least two params");
        }
    }

    fn get_params(function: LLVMValueRef) -> Vec<LLVMValueRef> {
        let params_count = unsafe {LLVMCountParams(function)};

        let mut params = Vec::<LLVMValueRef>::with_capacity(params_count as usize);
        unsafe {
            for i in 0..params_count {
                params.push(LLVMGetParam(function, i));
            }
        }

        params
    }
}

fn asserted_return(val: LLVMValueRef) -> LLVMValueRef {
    assert!(val != 0 as LLVMValueRef);
    val
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
            let mut error = 0 as *mut libc::c_char;
            let mut exec_engine = 0 as LLVMExecutionEngineRef;
            exec::LLVMCreateExecutionEngineForModule(&mut exec_engine,
                                                     module,
                                                     &mut error);

            assert!(exec_engine != 0 as exec::LLVMExecutionEngineRef);
            exec_engine
        };

        let mut env = LLVMEnvironment{context: context, builder: builder,
                                      module: module, engine: engine,
                                      named_values: HashMap::new()
        };

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
 }

declare double @atof(i8*) #2
"));

        env
    }

    #[allow(dead_code)]
    pub fn llvm_dump(&mut self) {
        unsafe {
            LLVMDumpModule(self.module);
        }
    }

    fn verify(&mut self) {
        unsafe {
            let mut error = 0 as *mut libc::c_char;
            analysis::LLVMVerifyModule(self.module,
                             analysis::LLVMVerifierFailureAction::LLVMAbortProcessAction,
                             &mut error);
        }

    }

    fn init_from_ir(&mut self, ir_code: String) {
        unsafe {
            let mut msg = 0 as *mut libc::c_char;
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

    fn get_fn(&mut self, name: &String) -> LLVMValueRef {
        unsafe {
            LLVMGetNamedFunction(self.module, cstring(name.clone()))
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

fn cstring_a(str: &'static str) -> *const libc::c_char {
    cstring(String::from(str))
}

fn cstring(str: String) -> *const libc::c_char {
    CString::new(str).unwrap().as_ptr()
}

fn buf_to_string(buf: *const libc::c_char) -> String {
    let str = unsafe {
        let slice = CStr::from_ptr(buf);
        str::from_utf8(slice.to_bytes()).unwrap()
    };

    String::from(str)
}

#[test]
fn demo_test() {
    demo();
}

pub fn demo() {
    let mut llvm_env = LLVMEnvironment::new();
    llvm_env.verify();

    let add = llvm_env.get_fn(&String::from("add"));
    let x: u64 = 2;
    let y: u64 = 3;

    let sum_result = unsafe {
        let int64 = LLVMInt64Type();

        let lx = exec::LLVMCreateGenericValueOfInt(int64, x, 1);
        let ly = exec::LLVMCreateGenericValueOfInt(int64, y, 1);

        let mut args = vec!(lx, ly);

        let res = exec::LLVMRunFunction(llvm_env.engine, add, 2, args.as_mut_ptr());

        exec::LLVMGenericValueToInt(res, 0)
    };

    println!("LLVM SUM RESULT {}", sum_result);

    assert_eq!(5, sum_result);
}
