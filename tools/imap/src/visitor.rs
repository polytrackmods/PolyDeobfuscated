use oxc_ast::ast;
use oxc_ast_visit::{Visit, walk};
use oxc_syntax::scope::ScopeFlags;

use crate::types::*;

#[derive(Debug, Default)]
pub struct IdentifierCollector {
    pub identifiers: Vec<IdentifierDeclarationType>,
    current_scope: u32,
}

impl IdentifierCollector {
    fn get_identifier_name(
        &self,
        name: String,
        declaration_type: &str,
    ) -> IdentifierDeclarationType {
        (
            name,
            self.current_scope,
            self.identifiers.len(),
            declaration_type.to_string(),
        )
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
                .push(self.get_identifier_name(identifier_name.to_string(), "variable"));
        }

        walk::walk_variable_declarator(self, declarator);
    }

    fn visit_function(&mut self, function: &ast::Function<'a>, flags: ScopeFlags) {
        if let Some(id) = &function.id {
            self.identifiers
                .push(self.get_identifier_name(id.name.to_string(), "function"));
        }

        for param in &function.params.items {
            if let Some(identifier_name) = param.pattern.get_identifier_name() {
                self.identifiers.push(
                    self.get_identifier_name(identifier_name.to_string(), "function_parameter"),
                );
            }
        }

        walk::walk_function(self, function, flags);
    }

    fn visit_arrow_function_expression(&mut self, arrow_fn: &ast::ArrowFunctionExpression<'a>) {
        for param in &arrow_fn.params.items {
            if let Some(identifier_name) = param.pattern.get_identifier_name() {
                self.identifiers.push(
                    self.get_identifier_name(
                        identifier_name.to_string(),
                        "arrow_function_parameter",
                    ),
                );
            }
        }

        walk::walk_arrow_function_expression(self, arrow_fn);
    }

    fn visit_class(&mut self, class: &ast::Class<'a>) {
        if let Some(id) = &class.id {
            self.identifiers
                .push(self.get_identifier_name(id.name.to_string(), "class"));
        }

        walk::walk_class(self, class);
    }

    fn visit_method_definition(&mut self, method: &ast::MethodDefinition<'a>) {
        if let Some(method_name) = method.key.name() {
            self.identifiers
                .push(self.get_identifier_name(method_name.to_string(), "method"));
        }

        for param in &method.value.params.items {
            if let Some(identifier_name) = param.pattern.get_identifier_name() {
                self.identifiers.push(
                    self.get_identifier_name(identifier_name.to_string(), "method_parameter"),
                );
            }
        }

        walk::walk_method_definition(self, method);
    }

    fn visit_property_definition(&mut self, property: &ast::PropertyDefinition<'a>) {
        if let Some(property_name) = property.key.name() {
            self.identifiers
                .push(self.get_identifier_name(property_name.to_string(), "property"));
        }

        walk::walk_property_definition(self, property);
    }
}
