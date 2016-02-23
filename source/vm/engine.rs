/**
 * Tagua VM
 *
 *
 * New BSD License
 *
 * Copyright © 2016-2016, Ivan Enderlin.
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *     * Redistributions of source code must retain the above copyright
 *       notice, this list of conditions and the following disclaimer.
 *     * Redistributions in binary form must reproduce the above copyright
 *       notice, this list of conditions and the following disclaimer in the
 *       documentation and/or other materials provided with the distribution.
 *     * Neither the name of the Hoa nor the names of its contributors may be
 *       used to endorse or promote products derived from this software without
 *       specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDERS AND CONTRIBUTORS BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 */

use super::LLVMRef;
use super::module::Module;

use libc::{c_char, c_uint, size_t};
use llvm::core::LLVMDisposeMessage;
use llvm::execution_engine::{
    LLVMCreateMCJITCompilerForModule,
    LLVMDisposeExecutionEngine,
    LLVMExecutionEngineRef,
    LLVMMCJITCompilerOptions
};
use llvm::target_machine::LLVMCodeModel;
use std::ffi::CStr;
use std::{mem, ptr};

#[derive(Clone)]
pub enum OptimizationLevel {
    NoOptimizations,
    Level1,
    Level2,
    Level3
}

pub enum CodeModel {
    Default,
    JITDefault,
    Kernel,
    Large,
    Medium,
    Small
}

pub struct Options {
    pub level     : OptimizationLevel,
    pub code_model: CodeModel
}

pub struct Engine {
    engine: LLVMExecutionEngineRef,
    owned : bool
}

impl Engine {
    pub fn new(module: &mut Module, options: &Options) -> Result<Engine, String> {
        let mut engine_options = LLVMMCJITCompilerOptions {
            OptLevel : options.level.clone() as c_uint,
            CodeModel: match options.code_model {
                CodeModel::Default    => LLVMCodeModel::LLVMCodeModelDefault,
                CodeModel::JITDefault => LLVMCodeModel::LLVMCodeModelJITDefault,
                CodeModel::Kernel     => LLVMCodeModel::LLVMCodeModelKernel,
                CodeModel::Large      => LLVMCodeModel::LLVMCodeModelLarge,
                CodeModel::Medium     => LLVMCodeModel::LLVMCodeModelMedium,
                CodeModel::Small      => LLVMCodeModel::LLVMCodeModelSmall
            },
            NoFramePointerElim: 0,
            EnableFastISel    : 1,
            MCJMM             : ptr::null_mut()
        };
        let engine_options_size = mem::size_of::<LLVMMCJITCompilerOptions>();
        let mut engine_ref      = 0 as LLVMExecutionEngineRef;
        let mut engine_error    = 0 as *mut c_char;

        let engine;

        unsafe {
            module.unown();
            engine = LLVMCreateMCJITCompilerForModule(
                &mut engine_ref,
                module.to_ref(),
                &mut engine_options,
                engine_options_size as u64,
                &mut engine_error
            );
        }

        if 0 == engine {
            let error;

            unsafe {
                error = CStr::from_ptr(engine_error).to_string_lossy().into_owned();
                LLVMDisposeMessage(engine_error);
            }

            Err(error)
        } else {
            Ok(
                Engine {
                    engine: engine_ref,
                    owned : true
                }
            )
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                LLVMDisposeExecutionEngine(self.engine);
            }
        }
    }
}

impl LLVMRef<LLVMExecutionEngineRef> for Engine {
    fn to_ref(&self) -> LLVMExecutionEngineRef {
        self.engine
    }
}


#[cfg(test)]
mod tests {
    use super::CodeModel;
    use super::Engine;
    use super::OptimizationLevel;
    use super::Options;
    use super::super::context::Context;
    use super::super::module::Module;

    #[test]
    fn case_ownership() {
        let context    = Context::new();
        let mut module = Module::new("foobar", &context);
        let result     = Engine::new(
            &mut module,
            &Options {
                level     : OptimizationLevel::NoOptimizations,
                code_model: CodeModel::Default
            }
        );

        match result {
            Ok(engine)
                => assert!(engine.owned),

            Err(error)
                => assert!(false)
        }
    }
}
