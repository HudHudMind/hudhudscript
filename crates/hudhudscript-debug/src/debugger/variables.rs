use super::{Debugger, ScopeVariable, WatchExpression};

impl Debugger {
    /// Inspect a single variable by name. Returns the value if the variable
    /// is present in the current scope snapshot.
    pub fn inspect(&self, name: &str) -> Option<&ScopeVariable> {
        self.scope_variables.iter().find(|v| v.name == name)
    }

    /// Returns all variables currently in scope.
    pub fn scope_variables(&self) -> &[ScopeVariable] {
        &self.scope_variables
    }

    /// Update the scope variables (called by the runtime when the debugger
    /// pauses so the user can inspect them).
    pub fn set_scope_variables(&mut self, variables: Vec<ScopeVariable>) {
        self.scope_variables = variables;
        for var in &self.scope_variables {
            if let Some(watch) = self.watch_expressions.get_mut(&var.name) {
                watch.last_value = Some(var.value.clone());
            }
        }
    }

    /// Clear scope variables (e.g. when execution resumes).
    pub fn clear_scope_variables(&mut self) {
        self.scope_variables.clear();
    }

    /// Add a watch expression. The runtime should evaluate this expression
    /// every time the debugger pauses and update its value via
    /// [`update_watch`].
    pub fn add_watch(&mut self, expression: String) {
        self.watch_expressions.insert(
            expression.clone(),
            WatchExpression {
                expression,
                last_value: None,
            },
        );
    }

    /// Remove a watch expression.
    pub fn remove_watch(&mut self, expression: &str) -> bool {
        self.watch_expressions.remove(expression).is_some()
    }

    /// Update the value of a watch expression.
    pub fn update_watch(&mut self, expression: &str, value: String) {
        if let Some(watch) = self.watch_expressions.get_mut(expression) {
            watch.last_value = Some(value);
        }
    }

    /// Returns all registered watch expressions and their last known values.
    pub fn watch_expressions(&self) -> Vec<&WatchExpression> {
        self.watch_expressions.values().collect()
    }

    /// Get the last value of a watch expression.
    pub fn get_watch_value(&self, expression: &str) -> Option<&str> {
        self.watch_expressions
            .get(expression)
            .and_then(|w| w.last_value.as_deref())
    }
}
