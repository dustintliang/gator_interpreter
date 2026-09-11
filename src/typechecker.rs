use std::collections::HashMap;
use crate::ast::{Expr, Program, Stmt, TopDecl};
use crate::types::{GatorType, parse_gator_type};

#[derive(Debug)]
pub struct TypeError {
    pub message: String
}

// Entry point — build type map from top-level declarations, then check each statement
pub fn check(program: &Program) -> Vec<TypeError> {
    let mut errors = Vec::new();
    let mut type_map: HashMap<String, GatorType> = HashMap::new();

    // gl_FragColor is a GLSL built-in output; Unknown means assignments are not Gator-checked
    type_map.insert("gl_FragColor".to_string(), GatorType::Unknown);

    for decl in &program.decls {
        match decl {
            TopDecl::Uniform {ty, name} => { type_map.insert(name.clone(), parse_gator_type(ty)); }
            TopDecl::Varying {ty, name} => { type_map.insert(name.clone(), parse_gator_type(ty)); }
            TopDecl::Precision {..} => {}
        }
    }

    for stmt in &program.stmts {
        check_stmt(stmt, &mut type_map, &mut errors, None);
    }

    errors
}

// frame_ctx is Some(frame) when inside an `in Frame { }` block
fn check_stmt(stmt: &Stmt, type_map: &mut HashMap<String, GatorType>, errors: &mut Vec<TypeError>, frame_ctx: Option<&str>) {
    match stmt {
        Stmt::Decl {ty, name, expr} => {
            if ty == "auto" {
                let inferred = infer(expr, type_map, errors);
                if let Some(ctx) = frame_ctx {
                    check_frame_ctx(&inferred, ctx, name, errors);
                }
                type_map.insert(name.clone(), inferred);
            } else {
                let declared = parse_gator_type(ty);
                let inferred = infer(expr, type_map, errors);
                if let Some(ctx) = frame_ctx {
                    check_frame_ctx(&declared, ctx, name, errors);
                }
                check_compatible(&declared, &inferred, name, errors);
                type_map.insert(name.clone(), declared);
            }
        }
        Stmt::Assign {name, expr} => {
            let inferred = infer(expr, type_map, errors);
            if let Some(declared) = type_map.get(name).cloned() {
                check_compatible(&declared, &inferred, name, errors);
            }
        }
        Stmt::SwizzleAssign {expr, ..} => { infer(expr, type_map, errors); }
        Stmt::In {frame, body} => {
            for s in body {
                check_stmt(s, type_map, errors, Some(frame.as_str()));
            }
        }
    }
}

// Infer the GatorType of an expression, returns Unknown when type cannot be determined
fn infer(expr: &Expr, type_map: &HashMap<String, GatorType>, errors: &mut Vec<TypeError>) -> GatorType {
    match expr {
        Expr::Float(_) => GatorType::Plain("float".to_string()),
        Expr::Int(_) => GatorType::Plain("int".to_string()),
        Expr::Ident(name) => type_map.get(name).cloned().unwrap_or(GatorType::Unknown),
        Expr::Swizzle {field, ..} => match field.len() {
            2 => GatorType::Plain("vec2".to_string()),
            3 => GatorType::Plain("vec3".to_string()),
            4 => GatorType::Plain("vec4".to_string()),
            _ => GatorType::Plain("float".to_string())
        },
        Expr::Call {name, args} => infer_call(name, args, type_map, errors),
        Expr::Cast {ty, ..} => parse_gator_type(ty),
        Expr::BinOp {op, left, right} => {
            let lt = infer(left, type_map, errors);
            let rt = infer(right, type_map, errors);
            match op {
                '+' | '-' => check_add(&lt, &rt, errors),
                '*' => check_mul(&lt, &rt, errors),
                _ => GatorType::Unknown
            }
        }
    }
}

fn infer_call(name: &str, args: &[Expr], type_map: &HashMap<String, GatorType>, _errors: &mut Vec<TypeError>) -> GatorType {
    // Evaluate args for type info: errors are discarded so pre-existing annotation issues inside call arguments don't surface as new false positives
    let mut sink = Vec::new();
    let arg_types: Vec<GatorType> = args.iter().map(|a| infer(a, type_map, &mut sink)).collect();
    match name {
        "vec3" => match arg_types.as_slice() {
            // vec3(float, float, float) or vec3(int, float, float) etc.
            [a, b, c] if is_scalar(a) && is_scalar(b) && is_scalar(c) => GatorType::Plain("vec3".to_string()),
            // vec3(vec4) — truncate
            [GatorType::Plain(a)] if a == "vec4" => GatorType::Plain("vec3".to_string()),
            _ => GatorType::Unknown
        },
        "vec4" => match arg_types.as_slice() {
            // vec4(float, float, float, float)
            [a, b, c, d] if is_scalar(a) && is_scalar(b) && is_scalar(c) && is_scalar(d) => GatorType::Plain("vec4".to_string()),
            // vec4(vec3, float) or vec4(vec2, float, float) — lift into homogeneous form
            [GatorType::Plain(a), b] if (a == "vec3" || a == "vec2") && is_scalar(b) => GatorType::Plain("vec4".to_string()),
            _ => GatorType::Unknown
        },
        "dot" | "length" | "distance" => GatorType::Plain("float".to_string()),
        "max" | "min" | "pow" | "clamp" | "mix" | "smoothstep" | "step" => GatorType::Plain("float".to_string()),
        // Frame-preserving: return same scheme+frames as first arg, subtype not tracked
        "normalize" | "reflect" | "faceforward" => {
            match arg_types.first() {
                Some(GatorType::Gator {scheme, frames, ..}) =>
                    GatorType::Gator {scheme: scheme.clone(), subtype: None, frames: frames.clone()},
                _ => GatorType::Unknown
            }
        }
        _ => GatorType::Unknown
    }
}

// True for types that can be used as scalar components (float or int literals)
fn is_scalar(t: &GatorType) -> bool {
    matches!(t, GatorType::Plain(s) if s == "float" || s == "int")
}

fn check_add(lt: &GatorType, rt: &GatorType, errors: &mut Vec<TypeError>) -> GatorType {
    match (lt, rt) {
        (GatorType::Unknown, _) | (_, GatorType::Unknown) => GatorType::Unknown,
        (GatorType::Plain(a), GatorType::Plain(b)) => {
            if a == b {
                // Both the same plain type (int+int, float+float, vec3+vec3, etc.)
                lt.clone()
            } else if a == "unannotated" || b == "unannotated" {
                // One side is a result of plain*plain — defer to the named side
                if b == "unannotated" { lt.clone() } else { rt.clone() }
            } else {
                errors.push(TypeError {message: format!("type mismatch in addition: {a} + {b}")});
                GatorType::Unknown
            }
        }
        _ if lt == rt => lt.clone(),
        _ => {
            errors.push(TypeError {message: format!("type mismatch in addition: {:?} + {:?}", lt, rt)});
            GatorType::Unknown
        }
    }
}

fn check_mul(lt: &GatorType, rt: &GatorType, errors: &mut Vec<TypeError>) -> GatorType {
    match (lt, rt) {
        (GatorType::Unknown, _) | (_, GatorType::Unknown) => GatorType::Unknown,
        // float * Gator -> Gator (scalar multiplication, frame preserved)
        (GatorType::Plain(s), GatorType::Gator {..}) if s == "float" => rt.clone(),
        (GatorType::Gator {..}, GatorType::Plain(s)) if s == "float" => lt.clone(),
        // Plain * Plain — both unannotated, produce unannotated result
        (GatorType::Plain(_), GatorType::Plain(_)) => GatorType::Plain("unannotated".to_string()),
        // Gator * Gator -> apply frame rules
        (GatorType::Gator {frames: lf, ..}, GatorType::Gator {frames: rf, ..}) => {
            if lf.len() == 2 && rf.len() == 1 {
                // Matrix<A,B> * vector<A> -> vector<B>
                if let GatorType::Gator {scheme: rs, subtype: rsub, ..} = rt {
                    if lf[0] == rf[0] {
                        GatorType::Gator {scheme: rs.clone(), subtype: rsub.clone(), frames: vec![lf[1].clone()]}
                    } else {
                        errors.push(TypeError {message: format!("frame mismatch in multiplication: matrix input '{}' but vector frame '{}'", lf[0], rf[0])});
                        GatorType::Unknown
                    }
                } else { unreachable!() }
            } else if lf.len() == 2 && rf.len() == 2 {
                // Matrix<A,B> * Matrix<B,C> -> Matrix<A,C>
                if let GatorType::Gator {scheme: ls, subtype: lsub, ..} = lt {
                    if lf[1] == rf[0] {
                        GatorType::Gator {scheme: ls.clone(), subtype: lsub.clone(), frames: vec![lf[0].clone(), rf[1].clone()]}
                    } else {
                        errors.push(TypeError {message: format!("frame mismatch in matrix * matrix: '{}' != '{}'", lf[1], rf[0])});
                        GatorType::Unknown
                    }
                } else { unreachable!() }
            } else {
                GatorType::Unknown
            }
        }
        // Annotated * unannotated (non-scalar) or vice versa -> error
        _ => {
            errors.push(TypeError {message: format!("cannot multiply {:?} by {:?}: cannot mix annotated and unannotated types", lt, rt)});
            GatorType::Unknown
        }
    }
}

fn check_compatible(declared: &GatorType, inferred: &GatorType, name: &str, errors: &mut Vec<TypeError>) {
    match (declared, inferred) {
        (_, GatorType::Unknown) | (GatorType::Unknown, _) => {}
        // Any plain type is assignable to any plain variable since GLSL handles that check
        (GatorType::Plain(_), GatorType::Plain(_)) => {}
        (a, b) if a == b => {}
        // Normalize/reflect return Gator with no subtype — check scheme+frames only
        (GatorType::Gator {scheme: ds, frames: df, ..}, GatorType::Gator {scheme: is, subtype: None, frames: inf})
            if ds == is && df == inf => {}
        // Plain named type assigned to Gator variable — idiomatic Gator: the annotation gives
        // the value its frame. Only "unannotated" (result of Plain*Plain) is disallowed.
        (GatorType::Gator {..}, GatorType::Plain(p)) if p != "unannotated" => {}
        _ => {
            errors.push(TypeError {message: format!("type mismatch for '{name}': declared {:?} but expression has type {:?}", declared, inferred)});
        }
    }
}

// Validate that a single-frame Gator type matches the enclosing in-block frame
fn check_frame_ctx(ty: &GatorType, frame: &str, name: &str, errors: &mut Vec<TypeError>) {
    if let GatorType::Gator {frames, ..} = ty {
        if frames.len() == 1 && frames[0] != frame {
            errors.push(TypeError {
                message: format!("'{name}' is in frame '{}' but declared inside 'in {frame}'", frames[0])
            });
        }
    }
}
