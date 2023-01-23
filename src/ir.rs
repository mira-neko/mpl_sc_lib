use std::collections::HashMap;

use super::ast_indexed::AstIndexed;
use mpl_vm::Instructions;

enum IrInst {
    Psh(f64),
    Pfa(u8),
    Pta(u8),
    Pek,
    Inp,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Max,
    Min,
    Swap(u8, u8),
    Label(String),
    Jmp(String),
    Jiz(String),
    Jnz(String),
}

enum IrInst2 {
    Psh(f64),
    Sap(u8),
    Pfa,
    Pta,
    Pek,
    Pop,
    Inp,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Max,
    Min,
    Jmp(String),
    Jiz(String),
    Jnz(String),
}

pub(super) struct Ir(Vec<IrInst>);

impl From<AstIndexed> for Ir {
    fn from(ai: AstIndexed) -> Ir {
        let mut ir = Vec::new();
        IrInst::new(&ai, &mut ir);
        Ir(ir)
    }
}

impl IrInst {
    fn new(ai: &AstIndexed, ir: &mut Vec<IrInst>) {
        match ai {
            AstIndexed::Root(inner) => inner.iter().for_each(|inst| IrInst::new(inst, ir)),
            AstIndexed::Value(val) => ir.push(IrInst::Psh(*val)),
            AstIndexed::Indx(id) => ir.push(IrInst::Pfa(id.clone())),
            AstIndexed::Assign(id, inner) => {
                IrInst::new(inner, ir);
                ir.push(IrInst::Pta(id.clone()))
            }
            AstIndexed::Input => ir.push(IrInst::Inp),
            AstIndexed::Print(inner) => inner.iter().for_each(|inst| {
                IrInst::new(inst, ir);
                ir.push(IrInst::Pek)
            }),
            AstIndexed::Add(inner1, inner2) => {
                IrInst::new(inner1, ir);
                IrInst::new(inner2, ir);
                ir.push(IrInst::Add)
            }
            AstIndexed::Sub(inner1, inner2) => {
                IrInst::new(inner1, ir);
                IrInst::new(inner2, ir);
                ir.push(IrInst::Sub)
            }
            AstIndexed::Mul(inner1, inner2) => {
                IrInst::new(inner1, ir);
                IrInst::new(inner2, ir);
                ir.push(IrInst::Mul)
            }
            AstIndexed::Div(inner1, inner2) => {
                IrInst::new(inner1, ir);
                IrInst::new(inner2, ir);
                ir.push(IrInst::Div)
            }
            AstIndexed::Mod(inner1, inner2) => {
                IrInst::new(inner1, ir);
                IrInst::new(inner2, ir);
                ir.push(IrInst::Mod)
            }
            AstIndexed::Max(inner1, inner2) => {
                IrInst::new(inner1, ir);
                IrInst::new(inner2, ir);
                ir.push(IrInst::Max)
            }
            AstIndexed::Min(inner1, inner2) => {
                IrInst::new(inner1, ir);
                IrInst::new(inner2, ir);
                ir.push(IrInst::Min)
            }
            AstIndexed::Swap(id0, id1) => ir.push(IrInst::Swap(id0.clone(), id1.clone())),
            AstIndexed::Label(id) => ir.push(IrInst::Label(id.clone())),
            AstIndexed::Goto(id) => ir.push(IrInst::Jmp(id.clone())),
            AstIndexed::GotoIf(id, inner) => {
                IrInst::new(inner, ir);
                ir.push(IrInst::Jiz(id.clone()))
            }
            AstIndexed::GotoIfNot(id, inner) => {
                IrInst::new(inner, ir);
                ir.push(IrInst::Jnz(id.clone()))
            }
        }
    }

    fn codegen(&self, prog: &mut Vec<IrInst2>, lblmgr: &mut HashMap<String, usize>) {
        match self {
            IrInst::Psh(val) => prog.push(IrInst2::Psh(*val)),
            IrInst::Pfa(id) => {
                prog.push(IrInst2::Sap(id.clone()));
                prog.push(IrInst2::Pfa)
            }
            IrInst::Pta(id) => {
                prog.push(IrInst2::Sap(id.clone()));
                prog.push(IrInst2::Pta)
            }
            IrInst::Inp => prog.push(IrInst2::Inp),
            IrInst::Pek => {
                prog.push(IrInst2::Pek);
                prog.push(IrInst2::Pop)
            }
            IrInst::Add => prog.push(IrInst2::Add),
            IrInst::Sub => prog.push(IrInst2::Sub),
            IrInst::Mul => prog.push(IrInst2::Mul),
            IrInst::Div => prog.push(IrInst2::Div),
            IrInst::Mod => prog.push(IrInst2::Mod),
            IrInst::Max => prog.push(IrInst2::Max),
            IrInst::Min => prog.push(IrInst2::Min),
            IrInst::Swap(id0, id1) => {
                prog.push(IrInst2::Sap(id0.clone()));
                prog.push(IrInst2::Pfa);
                prog.push(IrInst2::Sap(id1.clone()));
                prog.push(IrInst2::Pfa);
                prog.push(IrInst2::Sap(id0.clone()));
                prog.push(IrInst2::Pta);
                prog.push(IrInst2::Sap(id1.clone()));
                prog.push(IrInst2::Pta)
            }
            IrInst::Jmp(id) => prog.push(IrInst2::Jmp(id.clone())),
            IrInst::Jiz(id) => prog.push(IrInst2::Jiz(id.clone())),
            IrInst::Jnz(id) => prog.push(IrInst2::Jnz(id.clone())),
            IrInst::Label(id) => _ = lblmgr.insert(id.clone(), prog.len()),
        }
    }
}

impl Ir {
    pub(super) fn codegen(&self) -> Vec<Instructions> {
        let mut lblmgr = HashMap::new();
        let mut prog = Vec::new();
        for inst in &self.0 {
            inst.codegen(&mut prog, &mut lblmgr);
        }
        prog.iter().map(|inst| match inst {
            IrInst2::Psh(val) => Instructions::Psh(*val),
            IrInst2::Sap(id) => Instructions::Sap(*id),
            IrInst2::Pfa => Instructions::Pfa,
            IrInst2::Pta => Instructions::Pta,
            IrInst2::Pek => Instructions::Pek,
            IrInst2::Pop => Instructions::Pop,
            IrInst2::Inp => Instructions::Inp,
            IrInst2::Add => Instructions::Add,
            IrInst2::Sub => Instructions::Sub,
            IrInst2::Mul => Instructions::Mul,
            IrInst2::Div => Instructions::Div,
            IrInst2::Mod => Instructions::Mod,
            IrInst2::Max => Instructions::Max,
            IrInst2::Min => Instructions::Min,
            IrInst2::Jmp(id) => Instructions::Jmp(*lblmgr.get(id).unwrap()),
            IrInst2::Jiz(id) => Instructions::Jiz(*lblmgr.get(id).unwrap()),
            IrInst2::Jnz(id) => Instructions::Jnz(*lblmgr.get(id).unwrap()),
        }).collect()
    }
}
