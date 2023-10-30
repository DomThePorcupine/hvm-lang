use crate::term::{DefinitionBook, LetPat, Name, Term};
use hvmc::run::Val;
use std::collections::HashMap;

impl DefinitionBook {
  pub fn check_unbound_vars(&self) -> anyhow::Result<()> {
    for def in self.defs.values() {
      def.body.check_unbound_vars()?;
    }
    Ok(())
  }
}

impl Term {
  /// Checks that all variables are bound.
  /// Precondition: References have been resolved.
  pub fn check_unbound_vars(&self) -> anyhow::Result<()> {
    let mut globals = HashMap::new();
    check_uses(self, &mut HashMap::new(), &mut globals)?;

    // Check global vars
    for (nam, (declared, used)) in globals.into_iter() {
      if used && !declared {
        return Err(anyhow::anyhow!("Unbound unscoped variable '${nam}'"));
      }
    }
    Ok(())
  }
}

/// Scope has the number of times a name was declared in the current scope
/// Globals has how many times a global var name was declared and used.
pub fn check_uses<'a>(
  term: &'a Term,
  scope: &mut HashMap<&'a Name, Val>,
  globals: &mut HashMap<&'a Name, (bool, bool)>,
) -> anyhow::Result<()> {
  // TODO: Don't stop at the first error
  match term {
    Term::Lam { nam: Some(nam), bod } => {
      push_scope(nam, scope);
      check_uses(bod, scope, globals)?;
      pop_scope(nam, scope);
    }
    Term::Lam { nam: None, bod } => {
      check_uses(bod, scope, globals)?;
    }
    Term::Var { nam } => {
      if !scope.contains_key(nam) {
        return Err(anyhow::anyhow!("Unbound variable '{nam}'"));
      }
    }
    Term::Chn { nam, bod } => {
      globals.entry(nam).or_default().0 = true;
      check_uses(bod, scope, globals)?;
    }
    Term::Lnk { nam } => {
      globals.entry(nam).or_default().1 = true;
    }
    Term::Let { pat: LetPat::Var(nam), val, nxt } => {
      check_uses(val, scope, globals)?;
      push_scope(nam, scope);
      check_uses(nxt, scope, globals)?;
      pop_scope(nam, scope);
    }
    Term::Let { pat: LetPat::Tup(l_nam, r_nam), val, nxt } => {
      check_uses(val, scope, globals)?;

      if let Some(l_nam) = l_nam {
        push_scope(l_nam, scope);
      }
      if let Some(r_nam) = r_nam {
        push_scope(r_nam, scope);
      }

      check_uses(nxt, scope, globals)?;

      if let Some(l_nam) = l_nam {
        pop_scope(l_nam, scope);
      }
      if let Some(r_nam) = r_nam {
        pop_scope(r_nam, scope);
      }
    }
    Term::App { fun, arg } => {
      check_uses(fun, scope, globals)?;
      check_uses(arg, scope, globals)?;
    }
    Term::Match { cond, zero, succ } => {
      check_uses(cond, scope, globals)?;
      check_uses(zero, scope, globals)?;
      check_uses(succ, scope, globals)?;
    }
    Term::Dup { fst, snd, val, nxt } => {
      check_uses(val, scope, globals)?;
      if let Some(fst) = fst {
        push_scope(fst, scope)
      }
      if let Some(snd) = snd {
        push_scope(snd, scope);
      }
      check_uses(nxt, scope, globals)?;
      if let Some(snd) = snd {
        pop_scope(snd, scope);
      }
      if let Some(fst) = fst {
        pop_scope(fst, scope);
      }
    }
    Term::Sup { fst, snd } => {
      check_uses(fst, scope, globals)?;
      check_uses(snd, scope, globals)?;
    }
    Term::Ref { .. } | Term::Era => (),
    Term::Opx { fst, snd, .. } => {
      check_uses(fst, scope, globals)?;
      check_uses(snd, scope, globals)?;
    }
    Term::Num { .. } => (),
    Term::Tup { fst, snd } => {
      check_uses(fst, scope, globals)?;
      check_uses(snd, scope, globals)?;
    }
  }
  Ok(())
}

fn push_scope<'a>(nam: &'a Name, scope: &mut HashMap<&'a Name, Val>) {
  if let Some(n_declarations) = scope.get_mut(nam) {
    *n_declarations += 1;
  } else {
    scope.insert(nam, 1);
  }
}

fn pop_scope(nam: &Name, scope: &mut HashMap<&Name, Val>) {
  let n_declarations = scope.get_mut(nam).unwrap();
  *n_declarations -= 1;
  if *n_declarations == 0 {
    scope.remove(nam);
  }
}