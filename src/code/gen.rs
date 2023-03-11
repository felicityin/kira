use std::fs::File;
use std::io::{Result, Write};

use koopa::ir::entities::ValueData;
use koopa::ir::values::*;
use koopa::ir::{BasicBlock, FunctionData, Program, Value, ValueKind};

use crate::code::builder::AsmBuilder;
use crate::code::func::FunctionInfo;
use crate::code::info::{cur_func, cur_func_mut, ProgramInfo};
use crate::code::values::{asm_value, AsmValue};

/// Trait for generating RISC-V assembly.
pub trait GenerateToAsm<'p, 'i> {
    type Out;

    fn generate(&self, f: &mut File, info: &'i mut ProgramInfo<'p>) -> Result<Self::Out>;
}

/// Trait for generating RISC-V assembly (for values).
trait GenerateValueToAsm<'p, 'i> {
    type Out;

    fn generate(
        &self,
        f: &mut File,
        info: &'i mut ProgramInfo<'p>,
        v: &ValueData,
    ) -> Result<Self::Out>;
}

impl<'p, 'i> GenerateToAsm<'p, 'i> for Program {
    type Out = ();

    fn generate(&self, f: &mut File, info: &mut ProgramInfo) -> Result<Self::Out> {
        // generate global allocations
        for &value in self.inst_layout() {
            let data = self.borrow_value(value);
            let name = &data.name().as_ref().unwrap()[1..];
            info.insert_value(value, name.into());
            writeln!(f, "  .data")?;
            writeln!(f, "  .globl {name}")?;
            writeln!(f, "{name}:")?;
            data.generate(f, info)?;
            writeln!(f)?;
        }

        // generate functions
        for &func in self.func_layout() {
            info.set_cur_func(FunctionInfo::new(func));
            self.func(func).generate(f, info)?;
        }
        Ok(())
    }
}

impl<'p, 'i> GenerateToAsm<'p, 'i> for FunctionData {
    type Out = ();

    fn generate(&self, f: &mut File, info: &mut ProgramInfo) -> Result<Self::Out> {
        // skip declarations
        if self.layout().entry_bb().is_none() {
            return Ok(());
        }

        // allocation stack slots and log argument number
        let func = cur_func_mut!(info);
        for value in self.dfg().values().values() {
            // allocate stack slot
            if value.kind().is_local_inst() && !value.used_by().is_empty() {
                func.alloc_slot(value);
            }

            // log argument number
            if let ValueKind::Call(call) = value.kind() {
                func.log_arg_num(call.args().len());
            }
        }

        // generate basic block names
        for (&bb, data) in self.dfg().bbs() {
            // basic block parameters are not supported
            assert!(data.params().is_empty());
            func.log_bb_name(bb, data.name());
        }

        // generate prologue
        AsmBuilder::new(f, "t0").prologue(self.name(), func)?;

        // generate instructions in basic blocks
        for (bb, node) in self.layout().bbs() {
            let name = bb.generate(f, info)?;
            writeln!(f, "{name}:")?;
            for &inst in node.insts().keys() {
                self.dfg().value(inst).generate(f, info)?;
            }
        }
        Ok(())
    }
}  

impl<'p, 'i> GenerateToAsm<'p, 'i> for BasicBlock {
    type Out = &'i str;
  
    fn generate(&self, _: &mut File, info: &'i mut ProgramInfo) -> Result<Self::Out> {
        Ok(cur_func!(info).bb_name(*self))
    }
}

impl<'p, 'i> GenerateToAsm<'p, 'i> for ValueData {
    type Out = ();
  
    fn generate(&self, f: &mut File, info: &mut ProgramInfo) -> Result<Self::Out> {
        match self.kind() {
            ValueKind::Load(v) => v.generate(f, info, self),
            ValueKind::Store(v) => v.generate(f, info),
            ValueKind::Jump(v) => v.generate(f, info),
            ValueKind::Return(v) => v.generate(f, info),
            _ => Ok(()),
        }
    }
}

impl<'p, 'i> GenerateValueToAsm<'p, 'i> for Load {
    type Out = ();
  
    fn generate(&self, f: &mut File, info: &mut ProgramInfo, v: &ValueData) -> Result<Self::Out> {
        let src = self.src().generate(f, info)?;
        src.write_to(f, "t0")?;
        if src.is_ptr() {
            AsmBuilder::new(f, "t1").lw("t0", "t0", 0)?;
        }
        asm_value!(info, v).read_from(f, "t0", "t1")
    }
}

impl<'p, 'i> GenerateToAsm<'p, 'i> for Store {
    type Out = ();

    fn generate(&self, f: &mut File, info: &mut ProgramInfo) -> Result<Self::Out> {
        let sp_offset = cur_func!(info).sp_offset();
        let value = self.value().generate(f, info)?;
        if matches!(value, AsmValue::Arg(_)) {
            value.write_arg_to(f, "t0", sp_offset)?;
        } else {
            value.write_to(f, "t0")?;
        }

        let dest = self.dest().generate(f, info)?;
        if dest.is_ptr() {
            dest.write_to(f, "t1")?;
            AsmBuilder::new(f, "t2").sw("t0", "t1", 0)
        } else {
            dest.read_from(f, "t0", "t1")
        }
    }
}

impl<'p, 'i> GenerateToAsm<'p, 'i> for Jump {
    type Out = ();

    fn generate(&self, f: &mut File, info: &mut ProgramInfo) -> Result<Self::Out> {
        let label = self.target().generate(f, info)?;
        AsmBuilder::new(f, "t0").j(label)
    }
}  

impl<'p, 'i> GenerateToAsm<'p, 'i> for Return {
    type Out = ();

    fn generate(&self, f: &mut File, info: &mut ProgramInfo) -> Result<Self::Out> {
        if let Some(value) = self.value() {
            value.generate(f, info)?.write_to(f, "a0")?;
        }
        AsmBuilder::new(f, "t0").epilogue(cur_func!(info))
    }
}

impl<'p, 'i> GenerateToAsm<'p, 'i> for Value {
    type Out = AsmValue<'i>;

    fn generate(&self, _: &mut File, info: &'i mut ProgramInfo) -> Result<Self::Out> {
        if self.is_global() {
            Ok(AsmValue::Global(info.value(*self)))
        } else {
            let func = cur_func!(info);
            let value = info.program().func(func.func()).dfg().value(*self);
            Ok(match value.kind() {
                ValueKind::Integer(i) => AsmValue::Const(i.value()),
                ValueKind::FuncArgRef(i) => AsmValue::Arg(i.index()),
                _ => AsmValue::from(func.slot_offset(value)),
            })
        }
    }
}
