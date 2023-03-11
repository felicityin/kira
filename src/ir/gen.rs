
use koopa::ir::builder_traits::*;
use koopa::ir::{FunctionData, Program, Type};

use crate::ast::*;
use crate::ir::Result;
use crate::ir::func::FunctionInfo;
use crate::ir::scopes::{cur_func, cur_func_mut, Scopes};

/// Trait for generating Koopa IR program.
pub trait GenerateProgram<'ast> {
    type Out;

    fn generate(&'ast self, program: &mut Program, scopes: &mut Scopes) -> Result<Self::Out>;
}

impl<'ast> GenerateProgram<'ast> for CompUnit {
    type Out = ();
  
    fn generate(&'ast self, program: &mut Program, scopes: &mut Scopes) -> Result<Self::Out> {
        self.func_def.generate(program, scopes)?;
        Ok(())
    }
}

impl<'ast> GenerateProgram<'ast> for FuncDef {
    type Out = ();
  
    fn generate(&'ast self, program: &mut Program, scopes: &mut Scopes) -> Result<Self::Out> {
        // create new fucntion
        let params_ty = Vec::new();
        let ret_ty = self.func_type.generate(program, scopes)?;
        let mut data = FunctionData::new(format!("@{}", self.ident), params_ty, ret_ty);

        // generate entry/end/cur block
        let entry = data.dfg_mut().new_bb().basic_block(Some("%entry".into()));
        let end = data.dfg_mut().new_bb().basic_block(Some("%end".into()));
        let cur = data.dfg_mut().new_bb().basic_block(None);

        // generate return value
        let mut ret_val = None;
        if matches!(self.func_type, FuncType::Int) {
            let alloc = data.dfg_mut().new_value().alloc(Type::get_i32());
            data.dfg_mut().set_value_name(alloc, Some("%ret".into()));
            ret_val = Some(alloc);
        }

        // update function information
        let func = program.new_func(data);
        let mut info = FunctionInfo::new(func, entry, end, ret_val);
        info.push_bb(program, entry);
        if let Some(ret_val) = info.ret_val() {
            info.push_inst(program, ret_val);
        }
        info.push_bb(program, cur);

        // update scope
        scopes.cur_func = Some(info);

        // generate function body
        self.block.generate(program, scopes)?;

        // handle end basic block
        let mut info = scopes.cur_func.take().unwrap();
        info.seal_entry(program, cur);
        info.seal_func(program);
        Ok(())
    }
}  

impl<'ast> GenerateProgram<'ast> for FuncType {
    type Out = Type;
  
    fn generate(&'ast self, _: &mut Program, _scopes: &mut Scopes) -> Result<Self::Out> {
        Ok(match self {
            Self::Int => Type::get_i32(),
        })
    }
}

impl<'ast> GenerateProgram<'ast> for Block {
    type Out = ();
  
    fn generate(&'ast self, program: &mut Program, scopes: &mut Scopes) -> Result<Self::Out> {
        self.stmt.generate(program, scopes)?;
        Ok(())
    }
}

impl<'ast> GenerateProgram<'ast> for Stmt {
    type Out = ();
  
    fn generate(&'ast self, program: &mut Program, scopes: &mut Scopes) -> Result<Self::Out> {
        match self {
            Self::Return(s) => s.generate(program, scopes),
          }
    }
}

impl<'ast> GenerateProgram<'ast> for Return {
    type Out = ();
  
    fn generate(&'ast self, program: &mut Program, scopes: &mut Scopes) -> Result<Self::Out> {
        if let Some(ret_val) = cur_func!(scopes).ret_val() {
            // generate store
            let info = cur_func!(scopes);
            let value = cur_func!(scopes).new_value(program).integer(self.num);
            let store = info.new_value(program).store(value, ret_val);
            info.push_inst(program, store);
        }

        // jump to the end basic block
        let info = &mut cur_func_mut!(scopes);
        let jump = info.new_value(program).jump(info.end());
        info.push_inst(program, jump);

        // push new basic block
        let next = info.new_bb(program, None);
        info.push_bb(program, next);
        Ok(())
    }
}
