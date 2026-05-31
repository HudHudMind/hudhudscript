//! Type compatibility and unification predicates.

use super::Type;

impl Type {
    /// Check if this type is compatible with another type
    pub fn is_compatible_with(&self, other: &Type) -> bool {
        match (self, other) {
            // Any is compatible with everything
            (Type::Any, _) | (_, Type::Any) => true,

            // Same types are compatible
            (Type::String, Type::String) => true,
            (Type::Number, Type::Number) => true,
            (Type::Boolean, Type::Boolean) => true,
            (Type::Null, Type::Null) => true,
            (Type::Server, Type::Server) => true,

            // Array compatibility
            (Type::Array(a), Type::Array(b)) => a.is_compatible_with(b),

            // Function compatibility (contravariant params, covariant return)
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
                p1.len() == p2.len()
                    && p1
                        .iter()
                        .zip(p2.iter())
                        .all(|(a, b)| b.is_compatible_with(a))
                    && r1.is_compatible_with(r2)
            }

            // Promise compatibility
            (Type::Promise(a), Type::Promise(b)) => a.is_compatible_with(b),

            // Union type compatibility
            (Type::Union(types), other) => types.iter().any(|t| t.is_compatible_with(other)),
            (other, Type::Union(types)) => types.iter().any(|t| other.is_compatible_with(t)),

            // Parameterized type compatibility
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
                b1.is_compatible_with(b2)
                    && p1.len() == p2.len()
                    && p1
                        .iter()
                        .zip(p2.iter())
                        .all(|(a, b)| a.is_compatible_with(b))
            }

            // Generic types: equal name means compatible (alpha-equivalent type vars).
            (Type::Generic(a), Type::Generic(b)) => a == b,

            // Class compatibility
            (Type::Class { name: a, .. }, Type::Class { name: b, .. }) => a == b,
            (Type::Instance(a), Type::Instance(b)) => a == b,

            _ => false,
        }
    }

    /// Test whether two types unify under the given Hindley-Milner inference engine.
    ///
    /// Unlike `is_compatible_with`, this can resolve type variables (Generic):
    /// e.g., `Generic("T")` unifies with `Number` if no prior substitution
    /// conflicts. The engine accumulates substitutions as a side-effect.
    pub fn unifies_with(&self, other: &Type, inference: &mut crate::TypeInference) -> bool {
        inference.unify(self, other).is_ok()
    }
}
