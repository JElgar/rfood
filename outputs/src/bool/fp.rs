trait Identifier {}
struct Tik;
impl Identifier for Tik {}
struct Tok {
    pub n: Box<dyn Identifier>,
}
impl Identifier for Tok {}
trait Expr {
    fn search_pos(self: Box<Self>, ctx: Box<dyn Context>) -> Box<dyn Found>;
    fn search(self: Box<Self>) -> Box<dyn Found> {
        self.search_pos(Box::new(EmptyCtx {}))
    }
    fn search_neg(self: Box<Self>, ctx: Box<dyn Context>) -> Box<dyn Found>;
    fn evaluate(self: Box<Self>) -> Box<dyn Value> {
        self.search().evaluate_aux()
    }
}
struct EVar {
    pub n: Box<dyn Identifier>,
}
impl Expr for EVar {
    fn search_pos(self: Box<Self>, ctx: Box<dyn Context>) -> Box<dyn Found> {
        ctx.find_next(Box::new(ValPosVar { n: self.n }))
    }
    fn search_neg(self: Box<Self>, ctx: Box<dyn Context>) -> Box<dyn Found> {
        ctx.find_next(Box::new(ValNegVar { n: self.n }))
    }
}
struct ENot {
    pub e: Box<dyn Expr>,
}
impl Expr for ENot {
    fn search_pos(self: Box<Self>, ctx: Box<dyn Context>) -> Box<dyn Found> {
        self.e.search_neg(ctx)
    }
    fn search_neg(self: Box<Self>, ctx: Box<dyn Context>) -> Box<dyn Found> {
        Box::new(FoundRedex {
            r: Box::new(RedNot { e: self.e }),
            ctx,
        })
    }
}
struct EAnd {
    pub l: Box<dyn Expr>,
    pub r: Box<dyn Expr>,
}
impl Expr for EAnd {
    fn search_pos(self: Box<Self>, ctx: Box<dyn Context>) -> Box<dyn Found> {
        self.l.search_pos(Box::new(AndCtx1 { e: self.r, ctx }))
    }
    fn search_neg(self: Box<Self>, ctx: Box<dyn Context>) -> Box<dyn Found> {
        Box::new(FoundRedex {
            r: Box::new(RedAnd {
                l: self.l,
                r: self.r,
            }),
            ctx,
        })
    }
}
struct EOr {
    pub l: Box<dyn Expr>,
    pub r: Box<dyn Expr>,
}
impl Expr for EOr {
    fn search_pos(self: Box<Self>, ctx: Box<dyn Context>) -> Box<dyn Found> {
        self.l.search_pos(Box::new(OrCtx1 { e: self.r, ctx }))
    }
    fn search_neg(self: Box<Self>, ctx: Box<dyn Context>) -> Box<dyn Found> {
        Box::new(FoundRedex {
            r: Box::new(RedOr {
                l: self.l,
                r: self.r,
            }),
            ctx,
        })
    }
}
trait Value {
    fn as_expr(self: Box<Self>) -> Box<dyn Expr>;
}
struct ValPosVar {
    pub n: Box<dyn Identifier>,
}
impl Value for ValPosVar {
    fn as_expr(self: Box<Self>) -> Box<dyn Expr> {
        Box::new(EVar { n: self.n })
    }
}
struct ValNegVar {
    pub n: Box<dyn Identifier>,
}
impl Value for ValNegVar {
    fn as_expr(self: Box<Self>) -> Box<dyn Expr> {
        Box::new(ENot {
            e: Box::new(EVar { n: self.n }),
        })
    }
}
struct ValAnd {
    pub l: Box<dyn Value>,
    pub r: Box<dyn Value>,
}
impl Value for ValAnd {
    fn as_expr(self: Box<Self>) -> Box<dyn Expr> {
        Box::new(EAnd {
            l: self.l.as_expr(),
            r: self.r.as_expr(),
        })
    }
}
struct ValOr {
    pub l: Box<dyn Value>,
    pub r: Box<dyn Value>,
}
impl Value for ValOr {
    fn as_expr(self: Box<Self>) -> Box<dyn Expr> {
        Box::new(EOr {
            l: self.l.as_expr(),
            r: self.r.as_expr(),
        })
    }
}
trait Redex {
    fn eval(self: Box<Self>) -> Box<dyn Expr>;
}
struct RedNot {
    pub e: Box<dyn Expr>,
}
impl Redex for RedNot {
    fn eval(self: Box<Self>) -> Box<dyn Expr> {
        self.e
    }
}
struct RedAnd {
    pub l: Box<dyn Expr>,
    pub r: Box<dyn Expr>,
}
impl Redex for RedAnd {
    fn eval(self: Box<Self>) -> Box<dyn Expr> {
        Box::new(EOr {
            l: Box::new(ENot { e: self.l }),
            r: Box::new(ENot { e: self.r }),
        })
    }
}
struct RedOr {
    pub l: Box<dyn Expr>,
    pub r: Box<dyn Expr>,
}
impl Redex for RedOr {
    fn eval(self: Box<Self>) -> Box<dyn Expr> {
        Box::new(EAnd {
            l: Box::new(ENot { e: self.l }),
            r: Box::new(ENot { e: self.r }),
        })
    }
}
trait Found {
    fn evaluate_aux(self: Box<Self>) -> Box<dyn Value>;
}
struct FoundValue {
    pub v: Box<dyn Value>,
}
impl Found for FoundValue {
    fn evaluate_aux(self: Box<Self>) -> Box<dyn Value> {
        self.v
    }
}
struct FoundRedex {
    pub r: Box<dyn Redex>,
    pub ctx: Box<dyn Context>,
}
impl Found for FoundRedex {
    fn evaluate_aux(self: Box<Self>) -> Box<dyn Value> {
        self.ctx.substitute(self.r.eval()).evaluate()
    }
}
trait Context {
    fn find_next(self: Box<Self>, value: Box<dyn Value>) -> Box<dyn Found>;
    fn substitute(self: Box<Self>, expr: Box<dyn Expr>) -> Box<dyn Expr>;
}
struct EmptyCtx {}
impl Context for EmptyCtx {
    fn find_next(self: Box<Self>, value: Box<dyn Value>) -> Box<dyn Found> {
        Box::new(FoundValue { v: value })
    }
    fn substitute(self: Box<Self>, expr: Box<dyn Expr>) -> Box<dyn Expr> {
        return expr;
    }
}
struct AndCtx1 {
    e: Box<dyn Expr>,
    ctx: Box<dyn Context>,
}
impl Context for AndCtx1 {
    fn find_next(self: Box<Self>, value: Box<dyn Value>) -> Box<dyn Found> {
        self.e.search_pos(Box::new(AndCtx2 {
            v: value,
            ctx: self.ctx,
        }))
    }
    fn substitute(self: Box<Self>, expr: Box<dyn Expr>) -> Box<dyn Expr> {
        self.ctx.substitute(Box::new(EAnd { l: expr, r: self.e }))
    }
}
struct AndCtx2 {
    v: Box<dyn Value>,
    ctx: Box<dyn Context>,
}
impl Context for AndCtx2 {
    fn find_next(self: Box<Self>, value: Box<dyn Value>) -> Box<dyn Found> {
        self.ctx.find_next(Box::new(ValAnd {
            l: self.v,
            r: value,
        }))
    }
    fn substitute(self: Box<Self>, expr: Box<dyn Expr>) -> Box<dyn Expr> {
        self.ctx.substitute(Box::new(EAnd {
            l: self.v.as_expr(),
            r: expr,
        }))
    }
}
struct OrCtx1 {
    e: Box<dyn Expr>,
    ctx: Box<dyn Context>,
}
impl Context for OrCtx1 {
    fn find_next(self: Box<Self>, value: Box<dyn Value>) -> Box<dyn Found> {
        self.e.search_pos(Box::new(OrCtx2 {
            v: value,
            ctx: self.ctx,
        }))
    }
    fn substitute(self: Box<Self>, expr: Box<dyn Expr>) -> Box<dyn Expr> {
        self.ctx.substitute(Box::new(EOr { l: expr, r: self.e }))
    }
}
struct OrCtx2 {
    v: Box<dyn Value>,
    ctx: Box<dyn Context>,
}
impl Context for OrCtx2 {
    fn find_next(self: Box<Self>, value: Box<dyn Value>) -> Box<dyn Found> {
        self.ctx.find_next(Box::new(ValOr {
            l: self.v,
            r: value,
        }))
    }
    fn substitute(self: Box<Self>, expr: Box<dyn Expr>) -> Box<dyn Expr> {
        self.ctx.substitute(Box::new(EOr {
            l: self.v.as_expr(),
            r: expr,
        }))
    }
}
