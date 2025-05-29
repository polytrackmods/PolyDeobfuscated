use oxc::{
    ast::ast,
    ast_visit::{Visit, walk},
    span::Span,
    syntax,
};

use crate::types::*;

#[derive(Debug, Default)]
pub struct IdentifierCollector {
    pub identifiers: Vec<Identifier>,
    scope_head: usize,
    scope_id: usize,
    scope_stack: Vec<usize>,
    ignores: Vec<(String, usize)>,
    current_class_scope: Option<usize>,
}

impl IdentifierCollector {
    fn add_identifier(&mut self, name: String) {
        if self.ignores.iter().any(|(ignore_name, ignore_scope)| {
            ignore_name == &name && *ignore_scope == self.scope_id
        }) {
            return;
        }

        if self
            .identifiers
            .iter()
            .any(|id| id.name == name && id.scope_id == self.scope_id)
        {
            return;
        }

        self.identifiers.push(Identifier {
            name: name.clone(),
            scope_id: self.scope_id,
            id: self.identifiers.len(),
        });
    }

    fn add_identifier_with_scope(&mut self, name: String, scope_id: usize) {
        if self
            .ignores
            .iter()
            .any(|(ignore_name, ignore_scope)| ignore_name == &name && *ignore_scope == scope_id)
        {
            return;
        }

        if self
            .identifiers
            .iter()
            .any(|id| id.name == name && id.scope_id == scope_id)
        {
            return;
        }

        self.identifiers.push(Identifier {
            name: name.clone(),
            scope_id,
            id: self.identifiers.len(),
        });
    }
}

impl<'a> Visit<'a> for IdentifierCollector {
    fn enter_scope(
        &mut self,
        _: oxc::syntax::scope::ScopeFlags,
        _: &std::cell::Cell<Option<oxc::syntax::scope::ScopeId>>,
    ) {
        self.scope_head += 1;
        self.scope_id = self.scope_head;
        self.scope_stack.push(self.scope_id);
    }

    fn leave_scope(&mut self) {
        self.scope_stack.pop();
        self.scope_id = *self.scope_stack.last().unwrap_or(&0);
    }

    fn enter_node(&mut self, kind: oxc::ast::AstKind<'a>) {
        if let oxc::ast::AstKind::Class(_) = kind {
            self.current_class_scope = Some(self.scope_head + 1);
        }
    }

    fn leave_node(&mut self, kind: oxc::ast::AstKind<'a>) {
        if let oxc::ast::AstKind::Class(_) = kind {
            self.current_class_scope = None;
        }
    }

    fn visit_static_member_expression(&mut self, it: &ast::StaticMemberExpression<'a>) {
        if let ast::Expression::ThisExpression(_) = it.object {
            if let Some(class_scope) = self.current_class_scope {
                self.add_identifier_with_scope(it.property.name.to_string(), class_scope);
            } else {
                self.add_identifier(it.property.name.to_string());
            }
        }

        walk::walk_static_member_expression(self, it);
    }

    // for some reason, the scope does not get messed up for this, but does for binding identifiers (specifically in functions)
    fn visit_property_key(&mut self, it: &ast::PropertyKey<'a>) {
        self.add_identifier(it.name().unwrap().to_string());

        walk::walk_property_key(self, it);
    }

    fn visit_function(&mut self, it: &ast::Function<'a>, flags: oxc::syntax::scope::ScopeFlags) {
        if let Some(id) = &it.id {
            self.add_identifier(id.name.to_string());
            // TODO: find other cases where this is needed
            // this line seems weird, but because of the way walk works, it enters the scope, and then visits the binding identifier, which is wrong, so we ignore it
            self.ignores
                .push((id.name.to_string(), self.scope_head + 1));
        }

        walk::walk_function(self, it, flags);
    }

    fn visit_object_expression(&mut self, it: &ast::ObjectExpression<'a>) {
        // walk does not enter scope for object expressions, so we do it manually
        self.enter_scope(syntax::scope::ScopeFlags::Var, &std::cell::Cell::new(None));

        walk::walk_object_expression(self, it);
    }

    fn visit_binding_identifier(&mut self, it: &ast::BindingIdentifier<'a>) {
        self.add_identifier(it.name.to_string());

        walk::walk_binding_identifier(self, it);
    }
}

#[derive(Debug)]
pub struct Renamer {
    // (original span, new name) - requires some magic shenanigans to apply
    pub changes: Vec<(Span, String)>,
    mappings: Vec<Mapping>,
    scope_head: usize,
    scope_id: usize,
    scope_stack: Vec<usize>,
}

impl Renamer {
    pub fn new(mappings: Vec<Mapping>) -> Self {
        Self {
            changes: Vec::new(),
            mappings,
            scope_head: 0,
            scope_id: 0,
            scope_stack: Vec::new(),
        }
    }

    fn get_new_identifier(&self, name: &str) -> Option<String> {
        // reverse because we want the most recent definition of the identifier
        for scope_id in self.scope_stack.iter().rev() {
            if let Some(mapping) = self
                .mappings
                .iter()
                .find(|m| m.original.name == name && m.original.scope_id == *scope_id)
            {
                return Some(mapping.modified.name.clone());
            }
        }
        None
    }

    fn add_change(&mut self, span: Span, new_name: String) {
        if !self.changes.iter().any(|(existing_span, existing_name)| {
            existing_span == &span && existing_name == &new_name
        }) {
            self.changes.push((span, new_name));
        }
    }
}

impl<'a> Visit<'a> for Renamer {
    fn enter_scope(
        &mut self,
        _: oxc::syntax::scope::ScopeFlags,
        _: &std::cell::Cell<Option<oxc::syntax::scope::ScopeId>>,
    ) {
        self.scope_head += 1;
        self.scope_id = self.scope_head;
        self.scope_stack.push(self.scope_id);
    }

    fn leave_scope(&mut self) {
        self.scope_stack.pop();
        self.scope_id = *self.scope_stack.last().unwrap_or(&0);
    }

    fn visit_static_member_expression(&mut self, it: &ast::StaticMemberExpression<'a>) {
        if let ast::Expression::ThisExpression(_) = it.object
            && let Some(new_name) = self.get_new_identifier(it.property.name.as_ref()) {
                self.add_change(it.property.span, new_name);
            }

        walk::walk_static_member_expression(self, it);
    }

    fn visit_private_identifier(&mut self, it: &ast::PrivateIdentifier<'a>) {
        if let Some(new_name) = self.get_new_identifier(it.name.as_ref()) {
            self.add_change(it.span, format!("#{new_name}"));
        }

        walk::walk_private_identifier(self, it);
    }

    fn visit_property_key(&mut self, it: &ast::PropertyKey<'a>) {
        // TODO: add support for computed properties
        if let ast::PropertyKey::StaticIdentifier(static_identifier) = it {
            if let Some(new_name) = self.get_new_identifier(static_identifier.name.as_ref()) {
                self.add_change(static_identifier.span, new_name);
            }
        } else if let ast::PropertyKey::PrivateIdentifier(private_identifier) = it
            && let Some(new_name) = self.get_new_identifier(private_identifier.name.as_ref()) {
                self.add_change(private_identifier.span, format!("#{new_name}"));
            }

        walk::walk_property_key(self, it);
    }

    fn visit_function(&mut self, it: &ast::Function<'a>, flags: oxc::syntax::scope::ScopeFlags) {
        if let Some(id) = &it.id
            && let Some(new_name) = self.get_new_identifier(id.name.as_ref()) {
                self.add_change(id.span, new_name);
            }

        walk::walk_function(self, it, flags);
    }

    fn visit_object_expression(&mut self, it: &ast::ObjectExpression<'a>) {
        // walk does not enter scope for object expressions, so we do it manually
        self.enter_scope(syntax::scope::ScopeFlags::Var, &std::cell::Cell::new(None));

        walk::walk_object_expression(self, it);
    }

    fn visit_binding_identifier(&mut self, it: &ast::BindingIdentifier<'a>) {
        if let Some(new_name) = self.get_new_identifier(it.name.as_ref()) {
            self.add_change(it.span, new_name);
        }

        walk::walk_binding_identifier(self, it);
    }

    fn visit_identifier_reference(&mut self, it: &ast::IdentifierReference<'a>) {
        if let Some(new_name) = self.get_new_identifier(it.name.as_ref()) {
            self.add_change(it.span, new_name);
        }

        walk::walk_identifier_reference(self, it);
    }
}
