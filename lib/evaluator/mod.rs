#![allow(dead_code, unused_variables)] // TODO: Temporary

const DEBUG_STATE: bool = false;

use self::{
    generator::GenDefinition,
    structure::{
        AsciiStringDef, ByteNumberDef, Field, FieldDefinition, GroupDefinition,
        NumberFormat, TextNumberDef,
    },
};
use crate::{
    error::BajzelError,
    parser::{Expr, Ident, Literal, Program, Statement},
};
use std::collections::HashMap;

pub(crate) mod generator;
pub(crate) mod structure;

#[derive(Debug, Default)]
pub struct ProgramEnv {
    pub groups: HashMap<String, GroupDefinition>,
    pub gen: Option<GenDefinition>,
    cur_group: Option<String>,
    cur_field: Option<String>,
}

#[derive(Debug)]
pub enum Evaluator {
    Started(ProgramEnv),
    DefiningFields(ProgramEnv),
    DefiningFieldAttr(ProgramEnv),
    DefiningGenerator(ProgramEnv),
    Finished(ProgramEnv),
}

/// Evaluate program to an environment
///
pub fn evaluate_program(program: Program) -> Result<ProgramEnv, BajzelError> {
    let mut evaluator = Evaluator::Started(ProgramEnv::default());
    for statement in program.into_iter() {
        if DEBUG_STATE {
            println!(">>> {:?}", statement);
        }
        evaluator = evaluator.eval(statement)?;
        if DEBUG_STATE {
            println!("<<< {:#?}", evaluator);
            println!("----------------------------------------");
        }
    }
    match evaluator {
        Evaluator::Finished(ctx) => Ok(ctx),
        _ => Err(BajzelError::ProgramNotFinished),
    }
}

impl Evaluator {
    pub fn eval(self, statement: Statement) -> Result<Self, BajzelError> {
        match self {
            Evaluator::Started(ctx) => state_started(ctx, statement),
            Evaluator::DefiningFields(ctx) => {
                state_defining_fields(ctx, statement)
            }
            Evaluator::DefiningFieldAttr(ctx) => {
                state_defining_field_attr(ctx, statement)
            }
            Evaluator::DefiningGenerator(ctx) => {
                state_defining_generator(ctx, statement)
            }
            x => {
                eprintln!("####> {:?}", x);
                unimplemented!()
            }
        }
    }
}

fn state_started(
    mut ctx: ProgramEnv,
    statement: Statement,
) -> Result<Evaluator, BajzelError> {
    match statement {
        Statement::StartGroupDefinition(name) => {
            start_group_definition(&mut ctx, name)?;
            Ok(Evaluator::DefiningFields(ctx))
        }
        _ => syntax_err("At least one DEFINE section is required"),
    }
}

fn state_defining_fields(
    mut ctx: ProgramEnv,
    statement: Statement,
) -> Result<Evaluator, BajzelError> {
    ctx.cur_field = None;
    match statement {
        Statement::DefineConstField(literal, alias) => {
            define_const_field(literal, &mut ctx, alias)?;
            Ok(Evaluator::DefiningFields(ctx))
        }
        Statement::DefineVariableField(kind, alias) => {
            define_var_field(kind, &mut ctx, alias)?;
            Ok(Evaluator::DefiningFields(ctx))
        }
        Statement::MakeCurrentField(name) => {
            make_current_field(&mut ctx, name);
            Ok(Evaluator::DefiningFieldAttr(ctx))
        }
        Statement::StartGroupDefinition(name) => {
            start_group_definition(&mut ctx, name)?;
            Ok(Evaluator::DefiningFields(ctx))
        }
        Statement::StartGeneratorDefinition(name) => {
            start_generator_definition(&mut ctx, name)?;
            Ok(Evaluator::DefiningGenerator(ctx))
        }
        x => unimplemented!("state_defining_fields: {:?}", x),
    }
}

fn state_defining_field_attr(
    mut ctx: ProgramEnv,
    statement: Statement,
) -> Result<Evaluator, BajzelError> {
    match statement {
        Statement::UpdateField(ident, expr) => {
            update_current_field(&mut ctx, ident, expr)?;
            Ok(Evaluator::DefiningFieldAttr(ctx))
        }
        Statement::DefineConstField(literal, alias) => {
            define_const_field(literal, &mut ctx, alias)?;
            Ok(Evaluator::DefiningFields(ctx))
        }
        Statement::StartGroupDefinition(name) => {
            start_group_definition(&mut ctx, name)?;
            Ok(Evaluator::DefiningFields(ctx))
        }
        Statement::StartGeneratorDefinition(name) => {
            start_generator_definition(&mut ctx, name)?;
            Ok(Evaluator::DefiningGenerator(ctx))
        }
        x => unimplemented!("state_defining_field_attr: {:?}", x),
    }
}

fn state_defining_generator(
    mut ctx: ProgramEnv,
    statement: Statement,
) -> Result<Evaluator, BajzelError> {
    match statement {
        Statement::UpdateParam(name, expr) => {
            update_generator_param(&mut ctx, name, expr)?;
            Ok(Evaluator::DefiningGenerator(ctx))
        }
        Statement::Run => Ok(Evaluator::Finished(ctx)),
        x => unimplemented!("state_defining_generator: {:?}", x),
    }
}

fn start_group_definition(
    ctx: &mut ProgramEnv,
    name: Ident,
) -> Result<(), BajzelError> {
    // TODO: Cannot start group of the same name
    ctx.create_group(name);
    Ok(())
}

fn start_generator_definition(
    ctx: &mut ProgramEnv,
    name: Ident,
) -> Result<(), BajzelError> {
    ctx.create_generator(name)?;
    Ok(())
}

fn define_const_field(
    literal: Literal,
    ctx: &mut ProgramEnv,
    alias: Option<Ident>,
) -> Result<(), BajzelError> {
    let field_def = match literal {
        crate::parser::Literal::IntegerLiteral(x) => {
            FieldDefinition::TextNumber(TextNumberDef::new(NumberFormat::Int64))
        }
        crate::parser::Literal::StringLiteral(x) => {
            FieldDefinition::ConstString(x)
        }
        crate::parser::Literal::BytesLiteral(x) => {
            todo!("bytes literals not implemented")
        }
        crate::parser::Literal::Reserved(x) => {
            todo!("reserved literals not implemented")
        }
    };
    ctx.create_field(field_def, alias);
    Ok(())
}

fn define_var_field(
    kind: String,
    ctx: &mut ProgramEnv,
    alias: Option<Ident>,
) -> Result<(), BajzelError> {
    let mut field_def = None;
    if kind.starts_with("le_") || kind.starts_with("be_") {
        if let Ok(def) = ByteNumberDef::from_str(kind.as_str()) {
            field_def.replace(FieldDefinition::ByteNumber(def));
        }
    } else if kind == "string" {
        field_def.replace(FieldDefinition::AsciiString(AsciiStringDef::new()));
    } else if kind.starts_with('i') || kind.starts_with('u') {
        if let Ok(def) = TextNumberDef::from_str(kind.as_str()) {
            field_def.replace(FieldDefinition::TextNumber(def));
        }
    }
    let field_def = field_def.ok_or_else(|| {
        BajzelError::Syntax(String::from("Unsupported variable field type"))
    })?;
    ctx.create_field(field_def, alias);
    Ok(())
}

fn make_current_field(ctx: &mut ProgramEnv, name: Ident) {
    ctx.use_field(name);
}

fn update_current_field(
    ctx: &mut ProgramEnv,
    ident: Ident,
    expr: Expr,
) -> Result<(), BajzelError> {
    ctx.update_field(ident, expr)?;
    Ok(())
}

fn update_generator_param(
    ctx: &mut ProgramEnv,
    name: Ident,
    expr: Expr,
) -> Result<(), BajzelError> {
    ctx.update_param(name, expr)?;
    Ok(())
}

pub(crate) fn syntax_err<T>(msg: &str) -> Result<T, BajzelError> {
    Err(BajzelError::Syntax(msg.to_owned()))
}

impl ProgramEnv {
    ///
    ///
    fn create_group(&mut self, name: Ident) {
        self.groups
            .insert(name.to_string(), GroupDefinition::default());
        self.cur_group = Some(name.to_string());
        self.cur_field = None;
    }

    fn create_generator(&mut self, name: Ident) -> Result<(), BajzelError> {
        if self.gen.is_some() {
            syntax_err("single GENERATE section allowed")
        } else {
            self.gen = Some(GenDefinition::new(name.to_string()));
            Ok(())
        }
    }

    fn create_field(&mut self, def: FieldDefinition, alias: Option<Ident>) {
        let cur_group = self
            .cur_group
            .as_ref()
            .expect("current group should not be missing");
        let field = Field {
            def,
            alias: alias.map(|ident| ident.to_string()),
        };
        let group_def = self
            .groups
            .get_mut(cur_group)
            .expect("current group assigned only when group def of the same key is added");
        group_def.fields.push(field);
    }

    fn use_field(&mut self, name: Ident) {
        self.cur_field = Some(name.to_string());
    }

    fn update_field(
        &mut self,
        attr: Ident,
        expr: Expr,
    ) -> Result<(), BajzelError> {
        let expr = eval_expr(expr)?;
        let attr = attr.as_str();
        let field = self.get_field();
        match &mut field.def {
            FieldDefinition::ConstString(x) => {
                syntax_err("'string' type does not have any attributes")
            }
            FieldDefinition::TextNumber(def) => def.update(attr, expr),
            FieldDefinition::AsciiString(def) => def.update(attr, expr),
            FieldDefinition::ByteNumber(def) => def.update(attr, expr),
        }
    }

    fn get_field(&mut self) -> &mut Field {
        let cur_group =
            self.cur_group.as_ref().expect("current group must be set");
        let cur_field = self.cur_field.as_ref().expect("current field is set");
        let field = self
            .groups
            .get_mut(cur_group)
            .and_then(|x| {
                x.fields
                    .iter_mut()
                    .find(|x| x.alias.as_ref() == Some(cur_field))
            })
            .expect("Current group and current field must be set properly");

        field
    }

    fn update_param(
        &mut self,
        param: Ident,
        expr: Expr,
    ) -> Result<(), BajzelError> {
        let def = self.gen.as_mut().expect("GENERATE section is created");
        let expr = eval_expr(expr)?;
        let param = param.as_str();

        match param {
            "OUT_MIN" => def.set_min_output_len(expr),
            "OUT_MAX" => def.set_max_output_len(expr),
            "TERM" => def.set_term(expr),
            _ => syntax_err("unsupported generator parameter"),
        }
    }
}

fn eval_expr(expr: Expr) -> Result<Expr, BajzelError> {
    // Nothing to do for a now
    Ok(expr)
}

pub(crate) fn eval_expr_to_i64(expr: &Expr) -> Result<i64, BajzelError> {
    match expr {
        Expr::LiteralExpr(literal) => match literal {
            Literal::IntegerLiteral(value) => Ok(*value),
            Literal::StringLiteral(_) => todo!(),
            Literal::BytesLiteral(_) => todo!(),
            Literal::Reserved(_) => todo!(),
        },
        Expr::Group(_group) => Err(BajzelError::Expr(
            "eval_expr_to_i64: group expr not expected".to_owned(),
        )),
    }
}
