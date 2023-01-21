use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

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
            AstIndexed::Input => writeln!(f, "inp"),
            AstIndexed::Print(args) => args
                .iter()
                .try_for_each(|arg| writeln!(f, "{arg}pek\npop")),
            AstIndexed::Add(inner1, inner2) => inner1.fmt(f).and_then(|_| inner2.fmt(f)).and_then(|_| writeln!(f, "add")),
            AstIndexed::Sub(inner1, inner2) => inner1.fmt(f).and_then(|_| inner2.fmt(f)).and_then(|_| writeln!(f, "sub")),
            AstIndexed::Mul(inner1, inner2) => inner1.fmt(f).and_then(|_| inner2.fmt(f)).and_then(|_| writeln!(f, "mul")),
            AstIndexed::Div(inner1, inner2) => inner1.fmt(f).and_then(|_| inner2.fmt(f)).and_then(|_| writeln!(f, "div")),
            AstIndexed::Mod(inner1, inner2) => inner1.fmt(f).and_then(|_| inner2.fmt(f)).and_then(|_| writeln!(f, "mod")),
            AstIndexed::Max(inner1, inner2) => inner1.fmt(f).and_then(|_| inner2.fmt(f)).and_then(|_| writeln!(f, "max")),
            AstIndexed::Min(inner1, inner2) => inner1.fmt(f).and_then(|_| inner2.fmt(f)).and_then(|_| writeln!(f, "min")),
            AstIndexed::Swap(id0, id1) => writeln!(
                f,
                "sap {id0}\npfa\nsap {id1}\npfa\nsap {id0}\npta\nsap {id1}\npta"
            ),
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
                AstIndexed::Assign(
                    AstIndexed::assign(var_name, memmgr),
                    inner,
                )
            }
            Ast::Input => AstIndexed::Input,
            Ast::Print(args) => AstIndexed::Print(
                args.into_iter()
                    .map(|arg| AstIndexed::new(arg, memmgr.clone(), state.clone()))
                    .collect(),
            ),
            Ast::Add(inner1, inner2) => AstIndexed::Add(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, state))
            ),
            Ast::Sub(inner1, inner2) => AstIndexed::Sub(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, state))
            ),
            Ast::Mul(inner1, inner2) => AstIndexed::Mul(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, state))
            ),
            Ast::Div(inner1, inner2) => AstIndexed::Div(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, state))
            ),
            Ast::Mod(inner1, inner2) => AstIndexed::Mod(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, state))
            ),
            Ast::Max(inner1, inner2) => AstIndexed::Max(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, state))
            ),
            Ast::Min(inner1, inner2) => AstIndexed::Min(
                Box::new(AstIndexed::new(*inner1, memmgr.clone(), state.clone())),
                Box::new(AstIndexed::new(*inner2, memmgr, state))
            ),
            Ast::Swap(var1, var2) => AstIndexed::Swap(AstIndexed::assign(var1, memmgr.clone()), AstIndexed::assign(var2, memmgr)),
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
                let (cond, ty) = local_state._while.pop().expect("there are more elihws then whiles");
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
