use crate::ast::*;

pub struct Codegen {
    output: String,
    indent: usize,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            output: String::new(),
            indent: 0,
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        // standard C headers
        self.emit_line("#include <stdio.h>");
        self.emit_line("#include <stdlib.h>");
        self.emit_line("#include <string.h>");
        self.emit_line("#include <stdbool.h>");
        self.emit_line("");

        for item in &program.items {
            self.gen_top_level(item);
        }

        self.output.clone()
    }

    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }

    fn emit(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn emit_line(&mut self, s: &str) {
        let indent = self.indent_str();
        self.output.push_str(&format!("{}{}\n", indent, s));
    }

    fn gen_top_level(&mut self, item: &TopLevel) {
        match item {
            TopLevel::Function { return_ty, name, params, body } => {
                self.gen_function(return_ty, name, params, body);
            }
            TopLevel::StructDecl { .. } => todo!(),
            TopLevel::InterfaceDecl { .. } => todo!(),
            TopLevel::MethodDecl { .. } => todo!(),
        }
    }

    fn gen_function(&mut self, return_ty: &Type, name: &str, params: &[Param], body: &[Stmt]) {
        // in C, main must return int
        let ret = if name == "main" {
            "int".to_string()
        } else {
            self.gen_type(return_ty)
        };

        let params_str = if params.is_empty() {
            "void".to_string()
        } else {
            params.iter().map(|p| {
                let ty = self.gen_type(&p.ty);
                if p.by_ref || p.by_const_ref {
                    format!("{}* {}", ty, p.name)
                } else {
                    format!("{} {}", ty, p.name)
                }
            }).collect::<Vec<_>>().join(", ")
        };

        let noreturn = if matches!(return_ty, Type::Never) {
            "__attribute__((noreturn)) "
        } else {
            ""
        };

        self.emit_line(&format!("{}{} {}({}) {{", noreturn, ret, name, params_str));
        self.indent += 1;
        for stmt in body {
            self.gen_stmt(stmt);
        }
        // add return 0 for main
        if name == "main" {
            self.emit_line("return 0;");
        }
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
    }

    fn gen_type(&self, ty: &Type) -> String {
        match ty {
            Type::Int     => "int".to_string(),
            Type::Float   => "double".to_string(),
            Type::String  => "char*".to_string(),
            Type::Bool    => "bool".to_string(),
            Type::Void    => "void".to_string(),
            Type::Never   => "void".to_string(),
            Type::Nullable(inner) => self.gen_type(inner), // nullable is just a pointer in C
            Type::Array(_) => todo!(),
            Type::Null => "void*".to_string(),
        }
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { ty, name, value, is_const } => {
                let ty_str = match ty {
                    Some(t) => self.gen_type(t),
                    None => self.infer_type(value),
                };
                let const_str = if *is_const { "const " } else { "" };
                let val_str = self.gen_expr(value);
                self.emit_line(&format!("{}{} {} = {};", const_str, ty_str, name, val_str));
            }
            Stmt::Assign { name, value } => {
                let val_str = self.gen_expr(value);
                self.emit_line(&format!("{} = {};", name, val_str));
            }
            Stmt::Return(expr) => {
                match expr {
                    Some(e) => {
                        let val_str = self.gen_expr(e);
                        self.emit_line(&format!("return {};", val_str));
                    }
                    None => self.emit_line("return;"),
                }
            }
            Stmt::Expr(expr) => {
                let val_str = self.gen_expr(expr);
                self.emit_line(&format!("{};", val_str));
            }
            Stmt::If { condition, then_block, else_block } => {
                let cond_str = self.gen_expr(condition);
                self.emit_line(&format!("if ({}) {{", cond_str));
                self.indent += 1;
                for s in then_block {
                    self.gen_stmt(s);
                }
                self.indent -= 1;
                match else_block {
                    Some(else_stmts) => {
                        self.emit_line("} else {");
                        self.indent += 1;
                        for s in else_stmts {
                            self.gen_stmt(s);
                        }
                        self.indent -= 1;
                        self.emit_line("}");
                    }
                    None => self.emit_line("}"),
                }
            }
            Stmt::While { condition, body } => {
                let cond_str = self.gen_expr(condition);
                self.emit_line(&format!("while ({}) {{", cond_str));
                self.indent += 1;
                for s in body {
                    self.gen_stmt(s);
                }
                self.indent -= 1;
                self.emit_line("}");
            }
            Stmt::For { init, condition, step, body } => {
                // generate for loop header manually
                let cond_str = self.gen_expr(condition);
                let step_str = self.gen_stmt_inline(step);
                let init_str = self.gen_stmt_inline(init);
                self.emit_line(&format!("for ({} {}; {}) {{", init_str, cond_str, step_str));
                self.indent += 1;
                for s in body {
                    self.gen_stmt(s);
                }
                self.indent -= 1;
                self.emit_line("}");
            }
            Stmt::ForRange { ty, name, range, body } => {
                let ty_str = match ty {
                    Some(t) => self.gen_type(t),
                    None => "int".to_string(),
                };
                // range expr should be BinaryOp { Range | RangeInclusive }
                let (start, end, inclusive) = self.extract_range(range);
                let cmp = if inclusive { "<=" } else { "<" };
                self.emit_line(&format!(
                    "for ({} {} = {}; {} {} {}; {}++) {{",
                    ty_str, name, start, name, cmp, end, name
                ));
                self.indent += 1;
                for s in body {
                    self.gen_stmt(s);
                }
                self.indent -= 1;
                self.emit_line("}");
            }
        }
    }

    fn gen_stmt_inline(&mut self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::VarDecl { ty, name, value, is_const } => {
                let ty_str = match ty {
                    Some(t) => self.gen_type(t),
                    None => self.infer_type(value),
                };
                let const_str = if *is_const { "const " } else { "" };
                let val_str = self.gen_expr(value);
                format!("{}{} {} = {}", const_str, ty_str, name, val_str)
            }
            Stmt::Expr(e) => self.gen_expr(e),
            Stmt::Assign { name, value } => {
                format!("{} = {}", name, self.gen_expr(value))
            }
            _ => String::new(),
        }
    }

    fn extract_range(&self, expr: &Expr) -> (String, String, bool) {
        match expr {
            Expr::BinaryOp { op, left, right } => {
                let start = self.gen_expr_const(left);
                let end = self.gen_expr_const(right);
                let inclusive = matches!(op, BinaryOp::RangeInclusive);
                (start, end, inclusive)
            }
            _ => ("0".to_string(), "0".to_string(), false),
        }
    }

    fn gen_expr(&mut self, expr: &Expr) -> String {
        self.gen_expr_const(expr)
    }

    fn gen_expr_const(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Int(n)    => n.to_string(),
                Literal::Float(f)  => format!("{}", f),
                Literal::String(s) => format!("\"{}\"", s),
                Literal::Bool(b)   => if *b { "true".to_string() } else { "false".to_string() },
                Literal::Null      => "NULL".to_string(),
            },
            Expr::StringInterp(parts) => {
                let mut fmt_str = String::new();
                let mut args = vec![];

                for part in parts {
                    match part {
                        StringPart::Literal(s) => fmt_str.push_str(s),
                        StringPart::Expr(e) => {
                            fmt_str.push_str("%s");
                            args.push(self.gen_expr_const(e));
                        }
                    }
                }

                if args.is_empty() {
                    format!("\"{}\"", fmt_str)
                } else {
                    format!("\"{}\"", fmt_str) // returned as format string, args handled by caller
                }
            }
            Expr::Ident(name) => name.clone(),
            Expr::BinaryOp { op, left, right } => {
                let l = self.gen_expr_const(left);
                let r = self.gen_expr_const(right);
                let op_str = match op {
                    BinaryOp::Add          => "+",
                    BinaryOp::Sub          => "-",
                    BinaryOp::Mul          => "*",
                    BinaryOp::Div          => "/",
                    BinaryOp::Mod          => "%",
                    BinaryOp::Eq           => "==",
                    BinaryOp::Neq          => "!=",
                    BinaryOp::Lt           => "<",
                    BinaryOp::Gt           => ">",
                    BinaryOp::Lte          => "<=",
                    BinaryOp::Gte          => ">=",
                    BinaryOp::And          => "&&",
                    BinaryOp::Or           => "||",
                    BinaryOp::NullCoalesce => return format!("({} != NULL ? {} : {})", l, l, r),
                    BinaryOp::Pow          => return format!("pow({}, {})", l, r),
                    BinaryOp::Range
                    | BinaryOp::RangeInclusive => return format!("{}", r), // handled by ForRange
                };
                format!("({} {} {})", l, op_str, r)
            }
            Expr::UnaryOp { op, expr } => {
                let e = self.gen_expr_const(expr);
                match op {
                    UnaryOp::Not           => format!("(!{})", e),
                    UnaryOp::Neg           => format!("(-{})", e),
                    UnaryOp::PostIncrement => format!("{}++", e),
                    UnaryOp::PostDecrement => format!("{}--", e),
                }
            }
            Expr::Call { name, args } => {
                let args_str = args.iter()
                    .map(|a| self.gen_expr_const(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                // map Oxide built-ins to C equivalents
                match name.as_str() {
                    "println" => {
                        if args.is_empty() {
                            "printf(\"\\n\")".to_string()
                        } else {
                            let arg = &args[0];
                            match arg {
                                Expr::StringInterp(parts) => {
                                    let mut fmt_str = String::new();
                                    let mut printf_args = vec![];
                                    for part in parts {
                                        match part {
                                            StringPart::Literal(s) => fmt_str.push_str(s),
                                            StringPart::Expr(e) => {
                                                fmt_str.push_str("%s");
                                                printf_args.push(self.gen_expr_const(e));
                                            }
                                        }
                                    }
                                    let args_str = printf_args.join(", ");
                                    if args_str.is_empty() {
                                        format!("printf(\"{}\\n\")", fmt_str)
                                    } else {
                                        format!("printf(\"{}\\n\", {})", fmt_str, args_str)
                                    }
                                }
                                Expr::Literal(Literal::String(_)) => {
                                    format!("printf(\"%s\\n\", {})", self.gen_expr_const(arg))
                                }
                                _ => format!("printf(\"%d\\n\", {})", self.gen_expr_const(arg)),
                            }
                        }
                    }
                    "printf"  => format!("printf({})", args_str),
                    "sprintf" => format!("sprintf({})", args_str),
                    _         => format!("{}({})", name, args_str),
                }
            }
            Expr::FieldAccess { object, field } => {
                let obj = self.gen_expr_const(object);
                format!("{}.{}", obj, field)
            }
            Expr::MethodCall { object, method, args } => {
                let obj = self.gen_expr_const(object);
                let args_str = args.iter()
                    .map(|a| self.gen_expr_const(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}_{}({}, {})", obj, method, obj, args_str)
            }
            Expr::Index { array, index } => {
                let arr = self.gen_expr_const(array);
                let idx = self.gen_expr_const(index);
                format!("{}[{}]", arr, idx)
            }
        }
    }

    fn infer_type(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(Literal::Int(_))    => "int".to_string(),
            Expr::Literal(Literal::Float(_))  => "double".to_string(),
            Expr::Literal(Literal::String(_)) => "char*".to_string(),
            Expr::Literal(Literal::Bool(_))   => "bool".to_string(),
            Expr::Literal(Literal::Null)      => "void*".to_string(),
            _ => "int".to_string(), // fallback
        }
    }
}