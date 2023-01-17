use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use super::ast::Ast;

#[derive(Debug)]
pub(super) enum AstIndexed {
    Root(Vec<AstIndexed>),
    Value(f64),
    Indx(u8),
    Assign(u8, Box<AstIndexed>),
    Func(String, Vec<AstIndexed>),
    Label(u8),
    Goto(u8),
    GotoIf(u8, Box<AstIndexed>),
    GotoIfNot(u8, Box<AstIndexed>),
}

struct State {
    _while: Vec<(Box<AstIndexed>, bool)>,
}

impl From<Ast> for AstIndexed {
    fn from(ast: Ast) -> AstIndexed {
        let memmgr = Rc::new(RefCell::new(HashMap::new()));
        let state = Rc::new(RefCell::new(State { _while: Vec::new() }));
        AstIndexed::new(ast, memmgr, state)
    }
}

impl fmt::Display for AstIndexed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AstIndexed::Root(stmts) => stmts.iter().try_for_each(|stmt| stmt.fmt(f)),
            AstIndexed::Value(v) => writeln!(f, "psh {v}"),
            AstIndexed::Indx(id) => writeln!(f, "sap {id}\npfa"),
            AstIndexed::Assign(var_idx, inner) => writeln!(f, "{inner}sap {var_idx}\npta"),
            AstIndexed::Func(func_name, args) => match func_name.as_str() {
                "swap" => {
                    if let (AstIndexed::Indx(id0), AstIndexed::Indx(id1)) = (&args[0], &args[1]) {
                        writeln!(
                            f,
                            "sap {id0}\npfa\nsap {id1}\npfa\nsap {id0}\npta\nsap {id1}\npta"
                        )
                    } else {
                        panic!("swap cannot be done like that")
                    }
                }
                "print" => writeln!(f, "{}pek\npop", args[0]),
                "input" => writeln!(f, "inp"),
                _ => args
                    .iter()
                    .try_for_each(|arg| arg.fmt(f))
                    .and_then(|_| writeln!(f, "{func_name}")),
            },
            AstIndexed::Label(id) => writeln!(f, "sap {id}\nipta"),
            AstIndexed::Goto(id) => writeln!(f, "sap {id}\njmpa"),
            AstIndexed::GotoIf(id, cond) => writeln!(f, "{cond}sap {id}\njiza"),
            AstIndexed::GotoIfNot(id, cond) => writeln!(f, "{cond}sap {id}\njnza"),
        }
    }
}

impl AstIndexed {
    fn new(
        ast: Ast,
        memmgr: Rc<RefCell<HashMap<String, u8>>>,
        state: Rc<RefCell<State>>,
    ) -> AstIndexed {
        match ast {
            Ast::Root(inner) => AstIndexed::Root(
                inner
                    .into_iter()
                    .map(|inst| AstIndexed::new(inst, memmgr.clone(), state.clone()))
                    .collect(),
            ),
            Ast::Value(v) => AstIndexed::Value(v),
            Ast::Idnt(name) => AstIndexed::Indx(AstIndexed::get(name, memmgr)),
            Ast::Assign(var_name, inner) => AstIndexed::Assign(
                AstIndexed::assign(var_name, memmgr.clone()),
                Box::new(AstIndexed::new(*inner, memmgr, state)),
            ),
            Ast::Func(func_name, args) => AstIndexed::Func(
                func_name,
                args.into_iter()
                    .map(|arg| AstIndexed::new(arg, memmgr.clone(), state.clone()))
                    .collect(),
            ),
            Ast::Label(name) => AstIndexed::Label(AstIndexed::assign(name, memmgr)),
            Ast::Goto(name) => AstIndexed::Goto(AstIndexed::get(name, memmgr)),
            Ast::GotoIf(name, cond) => AstIndexed::GotoIf(
                AstIndexed::get(name, memmgr.clone()),
                Box::new(AstIndexed::new(*cond, memmgr, state)),
            ),
            Ast::GotoIfNot(name, cond) => AstIndexed::GotoIfNot(
                AstIndexed::get(name, memmgr.clone()),
                Box::new(AstIndexed::new(*cond, memmgr, state)),
            ),
            Ast::While(cond) => {
                let mut local_state = state.borrow_mut();
                let icond = AstIndexed::new(*cond, memmgr.clone(), state.clone());
                let name = format!("while_{icond:?}");
                local_state._while.push((Box::new(icond), true));
                AstIndexed::Label(AstIndexed::assign(name, memmgr))
            }
            Ast::WhileNot(cond) => {
                let mut local_state = state.borrow_mut();
                let icond = AstIndexed::new(*cond, memmgr.clone(), state.clone());
                let name = format!("while_{icond:?}");
                local_state._while.push((Box::new(icond), false));
                AstIndexed::Label(AstIndexed::assign(name, memmgr))
            }
            Ast::Elihw => {
                let mut local_state = state.borrow_mut();
                let (cond, ty) = local_state._while.pop().unwrap();
                let name = format!("while_{cond:?}");
                if ty {
                    AstIndexed::GotoIf(AstIndexed::get(name, memmgr), cond)
                } else {
                    AstIndexed::GotoIfNot(AstIndexed::get(name, memmgr), cond)
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
}
