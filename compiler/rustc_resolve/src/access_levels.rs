use rustc_ast::ast;
use rustc_ast::visit;
use rustc_ast::visit::Visitor;
use rustc_ast::EnumDef;
use rustc_ast::ForeignMod;
use rustc_ast_lowering::ResolverAstLowering;
use rustc_middle::middle::privacy::AccessLevel;

use crate::Resolver;

pub struct PrivacyVisitor<'r, 'a> {
    r: &'r mut Resolver<'a>,
    prev_level: Option<AccessLevel>,
}

impl<'r, 'a> PrivacyVisitor<'r, 'a> {
    pub fn new(r: &'r mut Resolver<'a>, root_level: Option<AccessLevel>) -> PrivacyVisitor<'r, 'a> {
        PrivacyVisitor { r, prev_level: root_level }
    }
}

impl<'r, 'ast> Visitor<'ast> for PrivacyVisitor<'ast, 'r> {
    fn visit_item(&mut self, item: &'ast ast::Item) {
        let inherited_item_level = match item.kind {
            // TODO Is this the correct behavior for those macros?
            ast::ItemKind::MacCall(..) | ast::ItemKind::MacroDef(..) => None,

            // Resolved in privacy when hir is available
            ast::ItemKind::Impl(..) => None,

            ast::ItemKind::ForeignMod(..) => self.prev_level,

            ast::ItemKind::ExternCrate(..)
            | ast::ItemKind::Use(..)
            | ast::ItemKind::Static(..)
            | ast::ItemKind::Const(..)
            | ast::ItemKind::Fn(..)
            | ast::ItemKind::Mod(..)
            | ast::ItemKind::GlobalAsm(..)
            | ast::ItemKind::TyAlias(..)
            | ast::ItemKind::Enum(..)
            | ast::ItemKind::Struct(..)
            | ast::ItemKind::Union(..)
            | ast::ItemKind::Trait(..)
            | ast::ItemKind::TraitAlias(..) => {
                if item.vis.kind.is_pub() {
                    self.prev_level
                } else {
                    None
                }
            }
        };

        let access_level = self.r.set_access_level(item.id, inherited_item_level);

        match item.kind {
            ast::ItemKind::ForeignMod(ForeignMod { ref items, .. }) => {
                for nested in items {
                    if nested.vis.kind.is_pub() {
                        self.r.set_access_level(nested.id, access_level);
                    }
                }
            }
            ast::ItemKind::Enum(EnumDef { ref variants }, _) => {
                for variant in variants {
                    let variant_level = self.r.set_access_level(variant.id, access_level);
                    if let Some(ctor_id) = variant.data.ctor_id() {
                        self.r.set_access_level(ctor_id, access_level);
                    }

                    for field in variant.data.fields() {
                        self.r.set_access_level(field.id, variant_level);
                    }
                }
            }
            ast::ItemKind::Struct(ref def, _) | ast::ItemKind::Union(ref def, _) => {
                if let Some(ctor_id) = def.ctor_id() {
                    self.r.set_access_level(ctor_id, access_level);
                }

                for field in def.fields() {
                    if field.vis.kind.is_pub() {
                        self.r.set_access_level(field.id, access_level);
                    }
                }
            }
            ast::ItemKind::Trait(ref trait_kind) => {
                for nested in trait_kind.4.iter() {
                    self.r.set_access_level(nested.id, access_level);
                }
            }
            ast::ItemKind::Impl(ref impl_kind) => {
                for nested in impl_kind.items.iter() {
                    if impl_kind.of_trait.is_some() || nested.vis.kind.is_pub() {
                        self.r.set_access_level(nested.id, access_level);
                    }
                }
            }
            ast::ItemKind::Mod(..) => {
                if access_level.is_some() {
                    self.r.set_exports_access_level(self.r.local_def_id(item.id));
                }
            }
            ast::ItemKind::ExternCrate(..)
            | ast::ItemKind::Use(..)
            | ast::ItemKind::Static(..)
            | ast::ItemKind::Const(..)
            | ast::ItemKind::GlobalAsm(..)
            | ast::ItemKind::TyAlias(..)
            | ast::ItemKind::TraitAlias(..)
            | ast::ItemKind::MacroDef(..)
            | ast::ItemKind::MacCall(..)
            | ast::ItemKind::Fn(..) => {}
        }

        let orig_level = std::mem::replace(&mut self.prev_level, access_level);
        visit::walk_item(self, item);
        self.prev_level = orig_level;
    }

    fn visit_block(&mut self, block: &'ast ast::Block) {
        let orig_level = std::mem::take(&mut self.prev_level);
        visit::walk_block(self, block);
        self.prev_level = orig_level;
    }
}
