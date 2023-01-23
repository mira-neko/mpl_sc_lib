use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::ast::Ast;

#[derive(Debug, Clone)]
pub(super) enum AstIndexed {
    Root(Vec<AstIndexed>),
    Value(f64),
    Indx(u8),
    Assign(u8, Box<AstIndexed>),
    Input,
    Print(Vec<AstIndexed>),
    Add(Box<AstIndexed>, Box<AstIndexed>),
    Sub(Box<AstIndexed>, Box<AstIndexed>),
    Mul(Box<AstIndexed>, Box<AstIndexed>),
    Div(Box<AstIndexed>, Box<AstIndexed>),
    Mod(Box<AstIndexed>, Box<AstIndexed>),
    Max(Box<AstIndexed>, Box<AstIndexed>),
    Min(Box<AstIndexed>, Box<AstIndexed>),
    Swap(u8, u8),
    Label(String),
    Goto(String),
    GotoIf(String, Box<AstIndexed>),
    GotoIfNot(String, Box<AstIndexed>),
}

struct State {
    _while: Vec<(Box<AstIndexed>, bool, usize)>,
    _if: Vec<(Box<AstIndexed>, bool, usize)>,
    whilec: usize,
    ifc: usize,
}

impl From<Ast> for AstIndexed {
    fn from(ast: Ast) -> AstIndexed {
        let memmgr = Rc::new(RefCell::new(HashMap::new()));
        let state = Rc::new(RefCell::new(State {
            _while: Vec::new(),
            _if: Vec::new(),
            whilec: 0,
            ifc: 0,
        }));
        AstIndexed::new(ast, memmgr, state)
    }
}

impl AstIndexed {
    fn new(
        ast: Ast,
        memmgr: Rc<RefCell<HashMap<String, u8>>>,
        state: Rc<RefCell<State>>,
    ) -> AstIndexed {
        match ast {
            Ast::Root(inner) => {
                let root = AstIndexed::Root(
                    inner
                        .into_iter()
                        .map(|inst| AstIndexed::new(inst, memmgr.clone(), state.clone()))
                        .collect(),
                );
                if !state.borrow()._while.is_empty() {
                    panic!("unclosed while");
                }
                root
            }
            Ast::Value(v) => AstIndexed::Value(v),
            Ast::Idnt(name) => AstIndexed::Indx(AstIndexed::get(name, memmgr)),
            Ast::Assign(var_name, inner) => {
                let inner = Box::new(AstIndexed::new(*inner, memmgr.clone(), state));
                AstIndexed::Assign(AstIndexed::assign(var_name, memmgr), inner)
            }
            Ast::Input => AstIndexed::Input,
            Ast::Print(args) => AstIndexed::Print(
                args.into_iter()
                    .map(|arg| AstIndexed::new(arg, memmgr.clone(), state.clone()))
                    .collect(),
            ),
            Ast::Add(inner1, inner2) => AstIndexed::Add(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, state)),
            ),
            Ast::Sub(inner1, inner2) => AstIndexed::Sub(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, state)),
            ),
            Ast::Mul(inner1, inner2) => AstIndexed::Mul(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, state)),
            ),
            Ast::Div(inner1, inner2) => AstIndexed::Div(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, state)),
            ),
            Ast::Mod(inner1, inner2) => AstIndexed::Mod(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, state)),
            ),
            Ast::Max(inner1, inner2) => AstIndexed::Max(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, state)),
            ),
            Ast::Min(inner1, inner2) => AstIndexed::Min(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, state)),
            ),
            Ast::Swap(var1, var2) => AstIndexed::Swap(
                AstIndexed::assign(var1, memmgr.clone()),
                AstIndexed::assign(var2, memmgr),
            ),
            Ast::Label(name) => AstIndexed::Label(name),
            Ast::Goto(name) => AstIndexed::Goto(name),
            Ast::GotoIf(name, cond) => {
                AstIndexed::GotoIf(name, Box::new(AstIndexed::new(*cond, memmgr, state)))
            }
            Ast::GotoIfNot(name, cond) => {
                AstIndexed::GotoIfNot(name, Box::new(AstIndexed::new(*cond, memmgr, state)))
            }
            Ast::While(cond) => {
                let mut local_state = state.borrow_mut();
                let icond = AstIndexed::new(*cond, memmgr, state.clone());
                let whilec = local_state.whilec;
                let name = format!("while_{icond:?}true{}", whilec);
                let name_end = format!("end_while_{icond:?}true{}", whilec);
                local_state
                    ._while
                    .push((Box::new(icond.clone()), true, whilec));
                local_state.whilec += 1;
                AstIndexed::Root(vec![
                    AstIndexed::GotoIfNot(name_end, Box::new(icond)),
                    AstIndexed::Label(name),
                ])
            }
            Ast::WhileNot(cond) => {
                let mut local_state = state.borrow_mut();
                let icond = AstIndexed::new(*cond, memmgr, state.clone());
                let whilec = local_state.whilec;
                let name = format!("while_{icond:?}false{}", whilec);
                let name_end = format!("end_while_{icond:?}false{}", whilec);
                local_state
                    ._while
                    .push((Box::new(icond.clone()), false, whilec));
                local_state.whilec += 1;
                AstIndexed::Root(vec![
                    AstIndexed::GotoIf(name_end, Box::new(icond)),
                    AstIndexed::Label(name),
                ])
            }
            Ast::If(cond) => {
                let mut local_state = state.borrow_mut();
                let icond = AstIndexed::new(*cond, memmgr, state.clone());
                let ifc = local_state.ifc;
                let name_end = format!("end_if_{icond:?}true{}", ifc);
                local_state
                    ._while
                    .push((Box::new(icond.clone()), true, ifc));
                local_state.ifc += 1;
                AstIndexed::GotoIfNot(name_end, Box::new(icond))
            }
            Ast::IfNot(cond) => {
                let mut local_state = state.borrow_mut();
                let icond = AstIndexed::new(*cond, memmgr, state.clone());
                let ifc = local_state.ifc;
                let name_end = format!("end_if_{icond:?}false{}", ifc);
                local_state
                    ._while
                    .push((Box::new(icond.clone()), false, ifc));
                local_state.ifc += 1;
                AstIndexed::GotoIf(name_end, Box::new(icond))
            }
            Ast::Elihw => {
                let mut local_state = state.borrow_mut();
                let (cond, ty, whilec) = local_state
                    ._while
                    .pop()
                    .expect("there are more elihws then whiles");
                let name = format!("while_{cond:?}{ty}{whilec}");
                let name_end = format!("end_while_{cond:?}{ty}{whilec}");
                if ty {
                    AstIndexed::Root(vec![
                        AstIndexed::GotoIf(name, cond),
                        AstIndexed::Label(name_end),
                    ])
                } else {
                    AstIndexed::Root(vec![
                        AstIndexed::GotoIfNot(name, cond),
                        AstIndexed::Label(name_end),
                    ])
                }
            }
            Ast::Fi => {
                let mut local_state = state.borrow_mut();
                let (cond, ty, ifc) = local_state
                    ._if
                    .pop()
                    .expect("there are more elihws then whiles");
                let name_end = format!("end_if_{cond:?}{ty}{ifc}");
                AstIndexed::Label(name_end)
            }
        }
    }

    fn assign(name: String, memmgr: Rc<RefCell<HashMap<String, u8>>>) -> u8 {
        let mut local_memmgr = memmgr.borrow_mut();
        if let Some(n) = local_memmgr.get(&name) {
            *n
        } else {
            let n = local_memmgr.len() as u8;
            local_memmgr.insert(name, n);
            n
        }
    }

    fn get(name: String, memmgr: Rc<RefCell<HashMap<String, u8>>>) -> u8 {
        let local_memmgr = memmgr.borrow_mut();
        if let Some(n) = local_memmgr.get(&name) {
            *n
        } else {
            panic!("uninitialized variable: {name}")
        }
    }
}
