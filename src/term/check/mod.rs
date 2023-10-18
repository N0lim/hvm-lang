use super::{DefId, DefinitionBook, Name, Term};

pub mod unbound_vars;

impl DefinitionBook {
  pub fn check_has_main(&self) -> anyhow::Result<DefId> {
    if let Some(main) = self.def_names.def_id(&Name::new("main")) {
      Ok(main)
    } else {
      Err(anyhow::anyhow!("File has no 'main' definition"))
    }
  }

  /// Check that a definition is not just a reference to itself
  pub fn check_ref_to_ref(&self) -> anyhow::Result<()> {
    for def in self.defs.values() {
      if let Term::Ref { .. } = def.body {
        return Err(anyhow::anyhow!(
          "Definition {} is just a reference to another definition",
          self.def_names.name(&def.def_id).unwrap()
        ));
      }
    }
    Ok(())
  }
}
