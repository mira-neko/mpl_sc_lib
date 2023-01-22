use std::collections::HashMap;

use super::ast_indexed::AstIndexed;
use mpl_vm::Instructions;

pub(super) enum IrInst {
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
    Label(usize),
    Jmp(usize),
    Jiz(usize),
    Jnz(usize),
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
            AstIndexed::Indx(id) => ir.push(IrInst::Pfa(*id)),
            AstIndexed::Assign(id, inner) => {
                IrInst::new(inner, ir);
                ir.push(IrInst::Pta(*id))
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
            AstIndexed::Swap(id0, id1) => ir.push(IrInst::Swap(*id0, *id1)),
            AstIndexed::Label(id) => ir.push(IrInst::Label(*id)),
            AstIndexed::Goto(id) => ir.push(IrInst::Jmp(*id)),
            AstIndexed::GotoIf(id, inner) => {
                IrInst::new(inner, ir);
                ir.push(IrInst::Jiz(*id))
            }
            AstIndexed::GotoIfNot(id, inner) => {
                IrInst::new(inner, ir);
                ir.push(IrInst::Jnz(*id))
            }
        }
    }

    fn codegen(&self, prog: &mut Vec<Instructions>, lblmgr: &mut HashMap<usize, usize>) {
        match *self {
            IrInst::Psh(val) => prog.push(Instructions::Psh(val)),
            IrInst::Pfa(id) => {
                prog.push(Instructions::Sap(id));
                prog.push(Instructions::Pfa)
            }
            IrInst::Pta(id) => {
                prog.push(Instructions::Sap(id));
                prog.push(Instructions::Pta)
            }
            IrInst::Inp => prog.push(Instructions::Inp),
            IrInst::Pek => {
                prog.push(Instructions::Pek);
                prog.push(Instructions::Pop)
            }
            IrInst::Add => prog.push(Instructions::Add),
            IrInst::Sub => prog.push(Instructions::Sub),
            IrInst::Mul => prog.push(Instructions::Mul),
            IrInst::Div => prog.push(Instructions::Div),
            IrInst::Mod => prog.push(Instructions::Mod),
            IrInst::Max => prog.push(Instructions::Max),
            IrInst::Min => prog.push(Instructions::Min),
            IrInst::Swap(id0, id1) => {
                prog.push(Instructions::Sap(id0));
                prog.push(Instructions::Pfa);
                prog.push(Instructions::Sap(id1));
                prog.push(Instructions::Pfa);
                prog.push(Instructions::Sap(id0));
                prog.push(Instructions::Pta);
                prog.push(Instructions::Sap(id1));
                prog.push(Instructions::Pta)
            }
            IrInst::Jmp(id) => prog.push(Instructions::Jmp(id)),
            IrInst::Jiz(id) => prog.push(Instructions::Jiz(id)),
            IrInst::Jnz(id) => prog.push(Instructions::Jnz(id)),
            IrInst::Label(id) => _ = lblmgr.insert(id, prog.len()),
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
        for inst in &mut prog {
            match *inst {
                Instructions::Jmp(id) => *inst = Instructions::Jmp(*lblmgr.get(&id).unwrap()),
                Instructions::Jiz(id) => *inst = Instructions::Jiz(*lblmgr.get(&id).unwrap()),
                Instructions::Jnz(id) => *inst = Instructions::Jnz(*lblmgr.get(&id).unwrap()),
                _ => (),
            }
        }
        prog
    }
}
