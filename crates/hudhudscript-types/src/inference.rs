//! Type inference engine (Hindley-Milner style)
//!
//! Implements real type unification with substitution tracking.
//! When unify(TypeVar, T) is called, the mapping is recorded.
//! apply() resolves all type variables to their unified types.

use crate::Type;
use std::collections::HashMap;

/// Type variable for inference
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVar(pub usize);

/// Type inference engine with substitution-based unification.
pub struct TypeInference {
    next_var: usize,
    /// Substitution map: TypeVar ID → resolved Type.
    /// Populated by `unify` when a Generic type variable is unified
    /// with a concrete type. Consumed by `apply`.
    substitutions: HashMap<String, Type>,
    /// Issue #1009: Generic constraint map.
    /// Maps generic param name (e.g. "T") to constraint type (e.g. Type::Number).
    /// When a generic is unified with a concrete type, the constraint is checked.
    constraints: HashMap<String, Type>,
}

impl Default for TypeInference {
    fn default() -> Self {
        Self::new()
    }
}

/// Issue #1009: Helper to resolve a constraint name string to a Type.
///
/// Shared by both checker and inference to avoid duplication (Kural 7).
pub fn resolve_constraint_name(name: &str) -> Option<Type> {
    match name {
        "Number" | "number" => Some(Type::Number),
        "String" | "string" => Some(Type::String),
        "Boolean" | "boolean" | "Bool" | "bool" => Some(Type::Boolean),
        "Null" | "null" => Some(Type::Null),
        "Any" | "any" => Some(Type::Any),
        _ => None, // Class/trait names need symbol table lookup — caller handles this
    }
}

impl TypeInference {
    /// Create a new type inference engine
    pub fn new() -> Self {
        Self {
            next_var: 0,
            substitutions: HashMap::new(),
            constraints: HashMap::new(),
        }
    }

    /// Issue #1009: Register a generic constraint.
    ///
    /// When `name` is later unified with a concrete type, the engine checks
    /// that the concrete type is compatible with `constraint_type`.
    pub fn add_constraint(&mut self, name: String, constraint_type: Type) {
        self.constraints.insert(name, constraint_type);
    }

    /// Issue #1009: Remove a generic constraint.
    pub fn remove_constraint(&mut self, name: &str) {
        self.constraints.remove(name);
    }

    /// Generate a fresh type variable name
    pub fn fresh_var(&mut self) -> TypeVar {
        let var = TypeVar(self.next_var);
        self.next_var += 1;
        var
    }

    /// Generate a fresh Generic type with a unique name
    pub fn fresh_type(&mut self) -> Type {
        let var = self.fresh_var();
        Type::Generic(format!("_T{}", var.0))
    }

    /// Unify two types, recording substitutions for type variables.
    ///
    /// When a `Type::Generic(name)` is unified with a concrete type,
    /// the substitution `name → concrete` is recorded. Subsequent calls
    /// to `apply()` will replace all occurrences of that generic.
    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<(), String> {
        // Apply existing substitutions first (path compression)
        let t1 = self.apply(t1);
        let t2 = self.apply(t2);

        match (&t1, &t2) {
            // Identical types — nothing to do
            _ if t1 == t2 => Ok(()),

            // Any unifies with everything (gradual typing escape hatch)
            (Type::Any, _) | (_, Type::Any) => Ok(()),

            // Generic binds to concrete type (the core of unification)
            (Type::Generic(name), other) | (other, Type::Generic(name)) => {
                // Occurs check: prevent infinite types like T = Array<T>
                if self.occurs_in(name, other) {
                    return Err(format!("Infinite type: {} occurs in {:?}", name, other));
                }
                // Issue #1009: Check generic constraint before binding
                if let Some(constraint) = self.constraints.get(name).cloned() {
                    if !matches!(other, Type::Any | Type::Generic(_))
                        && !other.is_compatible_with(&constraint)
                    {
                        return Err(format!(
                            "Generic constraint violated: {} requires {}, but got {}",
                            name,
                            constraint.type_name(),
                            other.type_name()
                        ));
                    }
                }
                self.substitutions.insert(name.clone(), other.clone());
                Ok(())
            }

            // Structural unification
            (Type::Array(a), Type::Array(b)) => self.unify(a, b),

            (Type::Promise(a), Type::Promise(b)) => self.unify(a, b),

            (
                Type::Function {
                    params: p1,
                    return_type: r1,
                },
                Type::Function {
                    params: p2,
                    return_type: r2,
                },
            ) => {
                if p1.len() != p2.len() {
                    return Err(format!(
                        "Function arity mismatch: {} vs {}",
                        p1.len(),
                        p2.len()
                    ));
                }
                for (a, b) in p1.iter().zip(p2.iter()) {
                    self.unify(a, b)?;
                }
                self.unify(r1, r2)
            }

            (Type::Object(props1), Type::Object(props2)) => {
                if props1.len() != props2.len() {
                    return Err(format!(
                        "Object property count mismatch: {} vs {}",
                        props1.len(),
                        props2.len()
                    ));
                }

                for (key, ty1) in props1 {
                    let ty2 = props2
                        .get(key)
                        .ok_or_else(|| format!("Missing object property: {}", key))?;
                    self.unify(ty1, ty2)?;
                }
                Ok(())
            }

            (
                Type::Parameterized {
                    base: b1,
                    params: p1,
                },
                Type::Parameterized {
                    base: b2,
                    params: p2,
                },
            ) => {
                self.unify(b1, b2)?;
                if p1.len() != p2.len() {
                    return Err(format!(
                        "Type parameter count mismatch: {} vs {}",
                        p1.len(),
                        p2.len()
                    ));
                }
                for (a, b) in p1.iter().zip(p2.iter()) {
                    self.unify(a, b)?;
                }
                Ok(())
            }

            // Union types: t1 must be compatible with at least one member of the union
            (_, Type::Union(types)) => {
                for t in types {
                    if self.unify(&t1, t).is_ok() {
                        return Ok(());
                    }
                }
                Err(format!("Cannot unify {:?} with union {:?}", t1, t2))
            }
            (Type::Union(types), _) => {
                for t in types {
                    if self.unify(t, &t2).is_ok() {
                        return Ok(());
                    }
                }
                Err(format!("Cannot unify union {:?} with {:?}", t1, t2))
            }

            (Type::Class { name: a, .. }, Type::Class { name: b, .. }) if a == b => Ok(()),
            (Type::Instance(a), Type::Instance(b)) if a == b => Ok(()),

            _ => Err(format!("Cannot unify {:?} with {:?}", t1, t2)),
        }
    }

    /// Check if a type variable occurs in a type (prevents infinite types).
    fn occurs_in(&self, var_name: &str, ty: &Type) -> bool {
        match ty {
            Type::Generic(name) => {
                if name == var_name {
                    return true;
                }
                // Check transitive substitutions
                if let Some(sub) = self.substitutions.get(name) {
                    return self.occurs_in(var_name, sub);
                }
                false
            }
            Type::Array(elem) => self.occurs_in(var_name, elem),
            Type::Promise(inner) => self.occurs_in(var_name, inner),
            Type::Function {
                params,
                return_type,
            } => {
                params.iter().any(|p| self.occurs_in(var_name, p))
                    || self.occurs_in(var_name, return_type)
            }
            Type::Object(props) => props.values().any(|v| self.occurs_in(var_name, v)),
            Type::Parameterized { base, params } => {
                self.occurs_in(var_name, base) || params.iter().any(|p| self.occurs_in(var_name, p))
            }
            Type::Union(types) => types.iter().any(|t| self.occurs_in(var_name, t)),
            _ => false,
        }
    }

    /// Apply all recorded substitutions to a type, resolving type variables.
    pub fn apply(&self, ty: &Type) -> Type {
        match ty {
            Type::Generic(name) => {
                if let Some(resolved) = self.substitutions.get(name) {
                    // Recursively apply (follow substitution chains)
                    self.apply(resolved)
                } else {
                    ty.clone()
                }
            }
            Type::Array(elem) => Type::Array(Box::new(self.apply(elem))),
            Type::Function {
                params,
                return_type,
            } => Type::Function {
                params: params.iter().map(|p| self.apply(p)).collect(),
                return_type: Box::new(self.apply(return_type)),
            },
            Type::Promise(inner) => Type::Promise(Box::new(self.apply(inner))),
            Type::Object(props) => {
                let new_props = props
                    .iter()
                    .map(|(k, v)| (k.clone(), self.apply(v)))
                    .collect();
                Type::Object(new_props)
            }
            Type::Parameterized { base, params } => Type::Parameterized {
                base: Box::new(self.apply(base)),
                params: params.iter().map(|p| self.apply(p)).collect(),
            },
            Type::Union(types) => Type::Union(types.iter().map(|t| self.apply(t)).collect()),
            _ => ty.clone(),
        }
    }

    /// Infer the most general type for a set of observed types.
    pub fn infer_general_type(&mut self, types: &[Type]) -> Type {
        if types.is_empty() {
            return Type::Any;
        }

        let first = &types[0];
        let all_same = types.iter().all(|t| t == first);

        if all_same {
            first.clone()
        } else {
            // Try to unify all types with a fresh variable
            let var = self.fresh_type();
            let mut can_unify = true;
            for t in types {
                if self.unify(&var, t).is_err() {
                    can_unify = false;
                    break;
                }
            }
            if can_unify {
                self.apply(&var)
            } else {
                // Fall back to union type
                Type::Union(types.to_vec())
            }
        }
    }

    /// Get the current substitution count (for diagnostics).
    pub fn substitution_count(&self) -> usize {
        self.substitutions.len()
    }
}
