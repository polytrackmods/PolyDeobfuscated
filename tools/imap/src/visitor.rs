use oxc_ast::ast;
use oxc_ast_visit::{Visit, walk};
use oxc_syntax::scope::ScopeFlags;

#[derive(Debug, Default)]
pub struct IdentifierCollector {
    pub identifiers: Vec<(String, u32, usize)>,
    current_scope: u32,
}

impl IdentifierCollector {
    fn get_identifier_name(&self, name: String) -> (String, u32, usize) {
        // We do this to avoid name collisions
        (name, self.current_scope, self.identifiers.len())
    }
}

impl<'a> Visit<'a> for IdentifierCollector {
    fn enter_scope(
        &mut self,
        _: ScopeFlags,
        _: &std::cell::Cell<Option<oxc_syntax::scope::ScopeId>>,
    ) {
        self.current_scope += 1;
    }

    fn leave_scope(&mut self) {
        self.current_scope -= 1;
    }

    fn visit_variable_declarator(&mut self, declarator: &ast::VariableDeclarator<'a>) {
        if let Some(identifier_name) = declarator.id.get_identifier_name() {
            self.identifiers
                .push(self.get_identifier_name(identifier_name.to_string()));
        }

        walk::walk_variable_declarator(self, declarator);
    }

    fn visit_function(&mut self, function: &ast::Function<'a>, flags: ScopeFlags) {
        // Add function name if it exists
        if let Some(id) = &function.id {
            self.identifiers
                .push(self.get_identifier_name(id.name.to_string()));
        }

        // Add parameter names
        for param in &function.params.items {
            if let Some(identifier_name) = param.pattern.get_identifier_name() {
                self.identifiers
                    .push(self.get_identifier_name(identifier_name.to_string()));
            }
        }

        walk::walk_function(self, function, flags);
    }

    fn visit_arrow_function_expression(&mut self, arrow_fn: &ast::ArrowFunctionExpression<'a>) {
        // Add parameter names
        for param in &arrow_fn.params.items {
            if let Some(identifier_name) = param.pattern.get_identifier_name() {
                self.identifiers
                    .push(self.get_identifier_name(identifier_name.to_string()));
            }
        }

        walk::walk_arrow_function_expression(self, arrow_fn);
    }

    fn visit_class(&mut self, class: &ast::Class<'a>) {
        // Add class name if it exists
        if let Some(id) = &class.id {
            self.identifiers
                .push(self.get_identifier_name(id.name.to_string()));
        }

        walk::walk_class(self, class);
    }

    fn visit_method_definition(&mut self, method: &ast::MethodDefinition<'a>) {
        // Add method name
        if let Some(method_name) = method.key.name() {
            self.identifiers
                .push(self.get_identifier_name(method_name.to_string()));
        }

        // Add parameter names
        for param in &method.value.params.items {
            if let Some(identifier_name) = param.pattern.get_identifier_name() {
                self.identifiers
                    .push(self.get_identifier_name(identifier_name.to_string()));
            }
        }

        walk::walk_method_definition(self, method);
    }

    fn visit_property_definition(&mut self, property: &ast::PropertyDefinition<'a>) {
        // Add property name
        if let Some(property_name) = property.key.name() {
            self.identifiers
                .push(self.get_identifier_name(property_name.to_string()));
        }

        walk::walk_property_definition(self, property);
    }
}
