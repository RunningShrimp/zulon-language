use crate::air::AirModule;
use crate::cap::CapTables;
use crate::effect::EffectTables;
use crate::hir::HirCrate;
use crate::ids::{CrateId, DefId, FileId};
use crate::mir::MirBody;
use crate::parse::AstFile;
use crate::resolve::DefMap;
use crate::ty::TyTables;
use std::sync::Arc;

pub trait Db {
    fn source_text(&self, file: FileId) -> Arc<str>;

    fn parse(&self, file: FileId) -> Arc<AstFile>;
    fn defmap(&self, crate_id: CrateId) -> Arc<DefMap>;
    fn hir(&self, crate_id: CrateId) -> Arc<HirCrate>;
    fn typeck(&self, crate_id: CrateId) -> Arc<TyTables>;
    fn effectck(&self, crate_id: CrateId) -> Arc<EffectTables>;
    fn capck(&self, crate_id: CrateId) -> Arc<CapTables>;
    fn mir(&self, def: DefId) -> Arc<MirBody>;
    fn air(&self, crate_id: CrateId) -> Arc<AirModule>;
}
