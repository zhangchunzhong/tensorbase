/*
*   Copyright (c) 2020 TensorBase, and its contributors
*   All rights reserved.

*   Licensed under the Apache License, Version 2.0 (the "License");
*   you may not use this file except in compliance with the License.
*   You may obtain a copy of the License at

*   http://www.apache.org/licenses/LICENSE-2.0

*   Unless required by applicable law or agreed to in writing, software
*   distributed under the License is distributed on an "AS IS" BASIS,
*   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*   See the License for the specific language governing permissions and
*   limitations under the License.
*/

use std::ffi::{c_void, CString};
use std::sync::Once;

static GLOBAL_INIT: Once = Once::new();

/**
 *
 */
pub struct Engine {
    eng: *const JitEng,
}

//TODO need to double chehck the thread safety of backgroud eng
unsafe impl Sync for Engine {}

impl Engine {
    pub fn new() -> Engine {
        unsafe {
            GLOBAL_INIT.call_once(|| jit_global_init());
            Engine {
                eng: jit_eng_new(0 as *mut *const ::std::os::raw::c_char),
            }
        }
    }

    fn new_with_includes<const N: usize>(
        inc_dirs: [&'static str; N],
    ) -> Engine {
        unsafe {
            GLOBAL_INIT.call_once(|| jit_global_init());
            Engine {
                eng: jit_eng_new(0 as *mut *const ::std::os::raw::c_char),
            }
        }
    }
    pub fn jitc(&self, src: &str) -> CompilationUnit {
        let err_out = std::ptr::null();
        unsafe {
            let cu0 = jit_eng_jitc(
                self.eng,
                CString::new(src).unwrap().as_ptr(),
                err_out,
            );
            return CompilationUnit { cu: cu0 };
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            jit_eng_release(self.eng);
        }
    }
}

//note: this kind will not drop its cu
pub struct GobalLibCompilationUnit {
    cu: *const JitCompUnit,
}

impl GobalLibCompilationUnit {
    pub fn init(engine: &Engine, src: &str) -> GobalLibCompilationUnit {
        let err_out = std::ptr::null();
        unsafe {
            let cu0 = jit_eng_jitc(
                engine.eng,
                CString::new(src).unwrap().as_ptr(),
                err_out,
            );
            return GobalLibCompilationUnit { cu: cu0 };
        }
    }
}

pub struct CompilationUnit {
    cu: *const JitCompUnit,
}

impl CompilationUnit {
    fn call_main(&self) -> i32 {
        let mut fn_main_ret = -1;
        let fn_args: *mut *mut c_void;
        unsafe {
            jit_cu_call(
                self.cu,
                CString::new("main").unwrap().as_ptr(),
                (&mut fn_main_ret) as *mut i32 as *mut c_void,
                0 as *mut *mut c_void,
            );
        }
        fn_main_ret
    }

    /**
     * base kernel "call convention":
     */
    pub fn call(&self) -> usize {
        let mut fn_main_ret = 0usize;
        unsafe {
            jit_cu_call(
                self.cu,
                CString::new("kernel").unwrap().as_ptr(),
                (&mut fn_main_ret) as *mut usize as *mut c_void,
                0 as *mut *mut c_void,
            );
        }
        fn_main_ret
    }

    //TODO call
    //FIXME arbitrary rust args into c?
}

impl Drop for CompilationUnit {
    fn drop(&mut self) {
        unsafe {
            jit_cu_release(self.cu);
        }
    }
}

/* automatically generated by rust-bindgen */

extern "C" {
    pub fn jit_global_init();
}
pub enum JitEng {}
extern "C" {
    pub fn jit_eng_new(
        inc_dirs: *mut *const ::std::os::raw::c_char,
    ) -> *mut JitEng;
}
extern "C" {
    pub fn jit_eng_release(eng: *const JitEng);
}
pub enum JitCompUnit {}
extern "C" {
    pub fn jit_eng_jitc(
        eng: *const JitEng,
        src: *const ::std::os::raw::c_char,
        err_out: *const ::std::os::raw::c_char,
    ) -> *const JitCompUnit;
}
extern "C" {
    pub fn jit_cu_call(
        cu: *const JitCompUnit,
        func_name: *const ::std::os::raw::c_char,
        func_ret: *mut c_void,
        func_args: *mut *mut c_void,
    );
}
extern "C" {
    pub fn jit_cu_release(cu: *const JitCompUnit);
}

#[cfg(test)]
mod unit_tests {
    use crate::Engine;
    use std::{ffi::c_void, time::Instant};
    #[test]
    fn test_engine_basic() {
        let lib_sources = include_str!("../tests/basic_check_c_lib.h");
        let fn_sources = include_str!("../tests/basic_check_c_src.c");
        // print!("lib_sources: {}", lib_sources);

        let eng1 = Engine::new();
        let eng2 = Engine::new_with_includes([""; 0]);

        let _ = eng1.jitc(lib_sources);
        let timer = Instant::now();
        let cu_fn = eng1.jitc(fn_sources);
        println!("{:?}", timer.elapsed());

        let ret = cu_fn.call_main();
        assert_eq!(ret, 0);
    }

    #[test]
    fn test_engine_basic2() {
        let fn_sources = include_str!("../tests/basic_check_c_src_2.c");
        // print!("lib_sources: {}", lib_sources);
        let eng1 = Engine::new();
        let timer = Instant::now();
        let cu_fn = eng1.jitc(fn_sources);
        println!("{:?}", timer.elapsed());

        let ret = cu_fn.call_main();
        assert_eq!(ret, 0);
    }

    // #[test]
    // fn test_engine_basic2() {
    //     let lib_sources = include_str!("../../runtime/ker_lib/ker_lib.h");
    //     let fn_sources = include_str!("../../runtime/ker_lib/ker_runner.c");
    //     // print!("lib_sources: {}", lib_sources);

    //     let eng1 = Engine::new();
    //     let eng2 = Engine::new_with_includes([""; 0]);

    //     let _ = eng1.jitc(lib_sources);
    //     let timer = Instant::now();
    //     let cu_fn = eng1.jitc(fn_sources);
    //     println!("{:?}", timer.elapsed());

    //     let ret = cu_fn.call_main();
    //     assert_eq!(ret, 0);
    // }

    // #[test]
    // fn test_engine_sum() {
    //     let fn_sources = include_str!("../../runtime/ker_lib/ker_lib.c");
    //     // print!("lib_sources: {}", lib_sources);

    //     let eng1 = Engine::new();
    //     // let eng2 = Engine::new_with_includes([""; 0]);

    //     // let cu_lib = eng1.jitc(lib_sources);
    //     let timer = Instant::now();
    //     let cu_fn = eng1.jitc(fn_sources);
    //     println!("{:?}", timer.elapsed());
    //     let mut agg_res = 0u64;
    //     let res_pp = &mut &mut agg_res as *mut _ as *mut *mut c_void;
    //     let ret = cu_fn.call(res_pp);
    //     assert_eq!(ret, 0);
    // }
}