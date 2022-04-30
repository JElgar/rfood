enum Identifier {
  Tik,
  Tok{n: Box<Identifier>},
}

enum Expr {
  EVar{n: Box<Identifier>},
  ENot{e: Box<Expr>},
  EAnd{l: Box<Expr>, r: Box<Expr>},
  EOr{l: Box<Expr>, r: Box<Expr>},
}

enum Value {
  ValPosVar{n: Box<Identifier>},
  ValNegVar{n: Box<Identifier>},
  ValAnd{l: Box<Value>, r: Box<Value>},
  ValOr{l: Box<Value>, r: Box<Value>},
}

enum Redex {
  RedNot{e: Box<Expr>},
  RedAnd{l: Box<Expr>, r: Box<Expr>},
  RedOr{l: Box<Expr>, r: Box<Expr>},
}

enum Found {
  FoundValue{v: Box<Value>},
  FoundRedex{r: Box<Redex>, ctx: Box<dyn Context>},
}

fn as_expr(value: Value) -> Expr {
    match value {
        Value::ValPosVar{n} => Expr::EVar{n: n},
        Value::ValNegVar{n} => Expr::ENot{e: Box::new(Expr::EVar{n: n})},
        Value::ValAnd{l, r} => Expr::EAnd{l: Box::new(as_expr(*l)), r: Box::new(as_expr(*r))},
        Value::ValOr{l, r} => Expr::EOr{l: Box::new(as_expr(*l)), r: Box::new(as_expr(*r))},
    }
}

fn eval(redex: Redex) -> Expr {
    match redex {
        Redex::RedNot{e} => *e,
        Redex::RedAnd{l, r} => Expr::EOr{l: Box::new(Expr::ENot{e: l}), r: Box::new(Expr::ENot{e: r})},
        Redex::RedOr{l, r} => Expr::EAnd{l: Box::new(Expr::ENot{e: l}), r: Box::new(Expr::ENot{e: r})}
  }
}

fn search(e: Expr) -> Found {
    search_pos(e, Box::new(EmptyCtx{}))
}

fn search_pos(expr: Expr, ctx: Box<dyn Context>) -> Found {
    match expr {
        Expr::EVar{n} => ctx.find_next(Value::ValPosVar{n: n}),
        Expr::ENot{e} => search_neg(*e, ctx),
        Expr::EAnd{l, r} => search_pos(*l, Box::new(AndCtx1{e: *r, ctx})),
        Expr::EOr{l, r} => search_pos(*l, Box::new(OrCtx1{e: *r, ctx})),
    }
}

fn search_neg(e: Expr, ctx: Box<dyn Context>) -> Found {
    match e {
        Expr::EVar{n} => ctx.find_next(Value::ValNegVar{n: n}),
        Expr::ENot{e} => Found::FoundRedex{r: Box::new(Redex::RedNot{e: e}), ctx},
        Expr::EAnd{l, r} => Found::FoundRedex{r: Box::new(Redex::RedAnd{l: l, r: r}), ctx},
        Expr::EOr{l, r} => Found::FoundRedex{r: Box::new(Redex::RedOr{l: l, r: r}), ctx}
  }
}

trait Context {
  fn find_next(self: Box<Self>, value: Value) -> Found;
  fn substitute(self: Box<Self>, expr: Expr) -> Expr;
}

struct EmptyCtx {}
impl Context for EmptyCtx {
    fn find_next(self: Box<Self>, value: Value) -> Found {
        Found::FoundValue{v: Box::new(value)}
    }

    fn substitute(self: Box<Self>, expr: Expr) -> Expr {
        return expr
    }
}

struct AndCtx1 {
    e: Expr,
    ctx: Box<dyn Context>,
}
impl Context for AndCtx1 {
    fn find_next(self: Box<Self>, value: Value) -> Found {
         search_pos(self.e, Box::new(AndCtx2{v: value, ctx: self.ctx}))
    }

    fn substitute(self: Box<Self>, expr: Expr) -> Expr {
        self.ctx.substitute(Expr::EAnd{l: Box::new(expr), r: Box::new(self.e)})
    }
}

struct AndCtx2 {
    v: Value,
    ctx: Box<dyn Context>,
}
impl Context for AndCtx2 {
    fn find_next(self: Box<Self>, value: Value) -> Found {
        self.ctx.find_next(Value::ValAnd{l: Box::new(self.v), r: Box::new(value)})
    }

    fn substitute(self: Box<Self>, expr: Expr) -> Expr {
        self.ctx.substitute(Expr::EAnd{l: Box::new(as_expr(self.v)), r: Box::new(expr)})
    }
}

struct OrCtx1 {
    e: Expr,
    ctx: Box<dyn Context>
}
impl Context for OrCtx1 {
    fn find_next(self: Box<Self>, value: Value) -> Found {
        search_pos(self.e, Box::new(OrCtx2{v: value, ctx: self.ctx}))
    }

    fn substitute(self: Box<Self>, expr: Expr) -> Expr {
        self.ctx.substitute(Expr::EOr{l: Box::new(expr), r: Box::new(self.e)})
    }
}

struct OrCtx2 {
    v: Value,
    ctx: Box<dyn Context>
}
impl Context for OrCtx2 {
    fn find_next(self: Box<Self>, value: Value) -> Found {
        self.ctx.find_next(Value::ValOr{l: Box::new(self.v), r: Box::new(value)})
    }

    fn substitute(self: Box<Self>, expr: Expr) -> Expr {
        self.ctx.substitute(Expr::EOr{l: Box::new(as_expr(self.v)), r: Box::new(expr)})
    }
}

fn evaluate(expr: Expr) -> Value {
    evaluate_aux(search(expr))
}

fn evaluate_aux(found: Found) -> Value {
    match found {
        Found::FoundValue{v} => *v,
        Found::FoundRedex{r, ctx} => evaluate(ctx.substitute(eval(*r)))
    }
}
