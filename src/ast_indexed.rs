use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::ast::Ast;

#[derive(Debug)]
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
    Label(usize),
    Goto(usize),
    GotoIf(usize, Box<AstIndexed>),
    GotoIfNot(usize, Box<AstIndexed>),
}

struct State {
    _while: Vec<(Box<AstIndexed>, bool)>,
}

impl From<Ast> for AstIndexed {
    fn from(ast: Ast) -> AstIndexed {
        let memmgr = Rc::new(RefCell::new(HashMap::new()));
        let lblmgr = Rc::new(RefCell::new(HashMap::new()));
        let state = Rc::new(RefCell::new(State { _while: Vec::new() }));
        AstIndexed::new(ast, memmgr, lblmgr, state)
    }
}

impl AstIndexed {
    fn new(
        ast: Ast,
        memmgr: Rc<RefCell<HashMap<String, u8>>>,
        lblmgr: Rc<RefCell<HashMap<String, usize>>>,
        state: Rc<RefCell<State>>,
    ) -> AstIndexed {
        match ast {
            Ast::Root(inner) => {
                let root = AstIndexed::Root(
                    inner
                        .into_iter()
                        .map(|inst| AstIndexed::new(inst, memmgr.clone(), lblmgr.clone(), state.clone()))
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
                let inner = Box::new(AstIndexed::new(*inner, memmgr.clone(), lblmgr, state));
                AstIndexed::Assign(
                    AstIndexed::assign(var_name, memmgr),
                    inner,
                )
            }
            Ast::Input => AstIndexed::Input,
            Ast::Print(args) => AstIndexed::Print(
                args.into_iter()
                    .map(|arg| AstIndexed::new(arg, memmgr.clone(), lblmgr.clone(), state.clone()))
                    .collect(),
            ),
            Ast::Add(inner1, inner2) => AstIndexed::Add(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), lblmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, lblmgr, state))
            ),
            Ast::Sub(inner1, inner2) => AstIndexed::Sub(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), lblmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, lblmgr, state))
            ),
            Ast::Mul(inner1, inner2) => AstIndexed::Mul(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), lblmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, lblmgr, state))
            ),
            Ast::Div(inner1, inner2) => AstIndexed::Div(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), lblmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, lblmgr, state))
            ),
            Ast::Mod(inner1, inner2) => AstIndexed::Mod(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), lblmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, lblmgr, state))
            ),
            Ast::Max(inner1, inner2) => AstIndexed::Max(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), lblmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, lblmgr, state))
            ),
            Ast::Min(inner1, inner2) => AstIndexed::Min(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), lblmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, lblmgr, state))
            ),
            Ast::Swap(var1, var2) => AstIndexed::Swap(AstIndexed::assign(var1, memmgr.clone()), AstIndexed::assign(var2, memmgr)),
            Ast::Label(name) => AstIndexed::Label(AstIndexed::assign_lbl(name, lblmgr)),
            Ast::Goto(name) => AstIndexed::Goto(AstIndexed::get_lbl(name, lblmgr)),
            Ast::GotoIf(name, cond) => AstIndexed::GotoIf(
                AstIndexed::get_lbl(name, lblmgr.clone()),
                Box::new(AstIndexed::new(*cond, memmgr, lblmgr, state)),
            ),
            Ast::GotoIfNot(name, cond) => AstIndexed::GotoIfNot(
                AstIndexed::get_lbl(name, lblmgr.clone()),
                Box::new(AstIndexed::new(*cond, memmgr, lblmgr, state)),
            ),
            Ast::While(cond) => {
                let mut local_state = state.borrow_mut();
                let icond = AstIndexed::new(*cond, memmgr, lblmgr.clone(), state.clone());
                let name = format!("while_{icond:?}");
                local_state._while.push((Box::new(icond), true));
                AstIndexed::Label(AstIndexed::assign_lbl(name, lblmgr))
            }
            Ast::WhileNot(cond) => {
                let mut local_state = state.borrow_mut();
                let icond = AstIndexed::new(*cond, memmgr, lblmgr.clone(), state.clone());
                let name = format!("while_{icond:?}");
                local_state._while.push((Box::new(icond), false));
                AstIndexed::Label(AstIndexed::assign_lbl(name, lblmgr))
            }
            Ast::Elihw => {
                let mut local_state = state.borrow_mut();
                let (cond, ty) = local_state._while.pop().expect("there are more elihws then whiles");
                let name = format!("while_{cond:?}");
                if ty {
                    AstIndexed::GotoIf(AstIndexed::get_lbl(name, lblmgr), cond)
                } else {
                    AstIndexed::GotoIfNot(AstIndexed::get_lbl(name, lblmgr), cond)
                }
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

    fn assign_lbl(name: String, lblmgr: Rc<RefCell<HashMap<String, usize>>>) -> usize {
        let mut local_lblmgr = lblmgr.borrow_mut();
        if let Some(n) = local_lblmgr.get(&name) {
            *n
        } else {
            let n = local_lblmgr.len();
            local_lblmgr.insert(name, n);
            n
        }
    }

    fn get_lbl(name: String, lblmgr: Rc<RefCell<HashMap<String, usize>>>) -> usize {
        let local_lblmgr = lblmgr.borrow_mut();
        if let Some(n) = local_lblmgr.get(&name) {
            *n
        } else {
            panic!("uninitialized label: {name}")
        }
    }
}
