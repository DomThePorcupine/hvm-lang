use super::{Book, Definition, Name, Pattern, Rule, Tag, Term};
use crate::maybe_grow;
use std::{fmt, ops::Deref};

/* Some aux structures for things that are not so simple to display */

pub struct DisplayFn<F: Fn(&mut fmt::Formatter) -> fmt::Result>(pub F);

impl<F: Fn(&mut fmt::Formatter) -> fmt::Result> fmt::Display for DisplayFn<F> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0(f)
  }
}

pub struct DisplayJoin<F, S>(pub F, pub S);

impl<F, I, S> fmt::Display for DisplayJoin<F, S>
where
  F: (Fn() -> I),
  I: IntoIterator,
  I::Item: fmt::Display,
  S: fmt::Display,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for (i, x) in self.0().into_iter().enumerate() {
      if i != 0 {
        self.1.fmt(f)?;
      }
      x.fmt(f)?;
    }
    Ok(())
  }
}

macro_rules! display {
  ($($x:tt)*) => {
    DisplayFn(move |f| write!(f, $($x)*))
  };
}

/* The actual display implementations */

impl fmt::Display for Term {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    maybe_grow(|| match self {
      Term::Lam { tag, nam, bod } => {
        write!(f, "{}λ{} {}", tag.display_padded(), var_as_str(nam), bod)
      }
      Term::Var { nam } => write!(f, "{nam}"),
      Term::Chn { tag, nam, bod } => {
        write!(f, "{}λ${} {}", tag.display_padded(), var_as_str(nam), bod)
      }
      Term::Lnk { nam } => write!(f, "${nam}"),
      Term::Let { nam, val, nxt } => {
        write!(f, "let {} = {}; {}", var_as_str(nam), val, nxt)
      }
      Term::Use { nam, val, nxt } => {
        let Some(nam) = nam else { unreachable!() };
        write!(f, "use {} = {}; {}", nam, val, nxt)
      }
      Term::Ref { nam: def_name } => write!(f, "{def_name}"),
      Term::App { tag, fun, arg } => {
        write!(f, "{}({} {})", tag.display_padded(), fun.display_app(tag), arg)
      }
      Term::Mat { arg, bnd, with, arms: rules } => {
        let with: Box<dyn std::fmt::Display> = if with.is_empty() {
          Box::new(display!(""))
        } else {
          Box::new(display!(" with {}", DisplayJoin(|| with, ", ")))
        };
        write!(
          f,
          "match {} = {}{} {{ {} }}",
          bnd.as_ref().unwrap(),
          arg,
          with,
          DisplayJoin(|| rules.iter().map(|rule| display!("{}: {}", var_as_str(&rule.0), rule.2)), "; "),
        )
      }
      Term::Swt { arg, bnd, with, pred: _, arms } => {
        let with: Box<dyn std::fmt::Display> = if with.is_empty() {
          Box::new(display!(""))
        } else {
          Box::new(display!(" with {}", DisplayJoin(|| with, ", ")))
        };
        let arms = DisplayJoin(
          || {
            arms.iter().enumerate().map(|(i, rule)| {
              display!("{}: {}", if i == arms.len() - 1 { "_".to_string() } else { i.to_string() }, rule)
            })
          },
          "; ",
        );
        write!(f, "switch {} = {}{} {{ {} }}", bnd.as_ref().unwrap(), arg, with, arms)
      }
      Term::Ltp { bnd, val, nxt } => {
        write!(f, "let ({}) = {}; {}", DisplayJoin(|| bnd.iter().map(var_as_str), ", "), val, nxt)
      }
      Term::Tup { els } => write!(f, "({})", DisplayJoin(|| els.iter(), ", "),),
      Term::Dup { tag, bnd, val, nxt } => {
        write!(f, "let {}{{{}}} = {}; {}", tag, DisplayJoin(|| bnd.iter().map(var_as_str), " "), val, nxt)
      }
      Term::Sup { tag, els } => {
        write!(f, "{}{{{}}}", tag, DisplayJoin(|| els, " "))
      }
      Term::Era => write!(f, "*"),
      Term::Num { val } => write!(f, "{val}"),
      Term::Nat { val } => write!(f, "#{val}"),
      Term::Str { val } => write!(f, "{val:?}"),
      Term::Opx { opr, fst, snd } => {
        write!(f, "({} {} {})", opr, fst, snd)
      }
      Term::Lst { els } => write!(f, "[{}]", DisplayJoin(|| els.iter(), ", "),),
      Term::Err => write!(f, "<Invalid>"),
    })
  }
}

impl fmt::Display for Tag {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Tag::Named(name) => write!(f, "#{name}"),
      Tag::Numeric(num) => write!(f, "#{num}"),
      Tag::Auto => Ok(()),
      Tag::Static => Ok(()),
    }
  }
}

impl fmt::Display for Pattern {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Pattern::Var(None) => write!(f, "*"),
      Pattern::Var(Some(nam)) => write!(f, "{nam}"),
      Pattern::Ctr(nam, pats) => {
        write!(f, "({}{})", nam, DisplayJoin(|| pats.iter().map(|p| display!(" {p}")), ""))
      }
      Pattern::Num(num) => write!(f, "{num}"),
      Pattern::Tup(pats) => write!(f, "({})", DisplayJoin(|| pats, ", ")),
      Pattern::Lst(pats) => write!(f, "[{}]", DisplayJoin(|| pats, ", ")),
      Pattern::Str(str) => write!(f, "\"{str}\""),
    }
  }
}

impl Rule {
  pub fn display<'a>(&'a self, def_name: &'a Name) -> impl fmt::Display + 'a {
    display!(
      "({}{}) = {}",
      def_name,
      DisplayJoin(|| self.pats.iter().map(|x| display!(" {x}")), ""),
      self.body
    )
  }
}

impl fmt::Display for Definition {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", DisplayJoin(|| self.rules.iter().map(|x| x.display(&self.name)), "\n"))
  }
}

impl fmt::Display for Book {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", DisplayJoin(|| self.defs.values(), "\n\n"))
  }
}

impl fmt::Display for Name {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl Term {
  fn display_app<'a>(&'a self, tag: &'a Tag) -> impl fmt::Display + 'a {
    maybe_grow(|| {
      DisplayFn(move |f| match self {
        Term::App { tag: tag2, fun, arg } if tag2 == tag => {
          write!(f, "{} {}", fun.display_app(tag), arg)
        }
        _ => write!(f, "{}", self),
      })
    })
  }
}

impl Tag {
  pub fn display_padded(&self) -> impl fmt::Display + '_ {
    DisplayFn(move |f| match self {
      Tag::Named(name) => write!(f, "#{name} "),
      Tag::Numeric(num) => write!(f, "#{num} "),
      Tag::Auto => Ok(()),
      Tag::Static => Ok(()),
    })
  }
}

fn var_as_str(nam: &Option<Name>) -> &str {
  nam.as_ref().map_or("*", Name::deref)
}

/* Pretty printing  */

impl Book {
  pub fn display_pretty(&self) -> impl fmt::Display + '_ {
    display!("{}", DisplayJoin(|| self.defs.values().map(|def| def.display_pretty()), "\n\n"))
  }
}

impl Definition {
  pub fn display_pretty(&self) -> impl fmt::Display + '_ {
    display!("{}", DisplayJoin(|| self.rules.iter().map(|x| x.display_pretty(&self.name)), "\n"))
  }
}

impl Rule {
  pub fn display_pretty<'a>(&'a self, def_name: &'a Name) -> impl fmt::Display + 'a {
    display!(
      "({}{}) =\n  {}",
      def_name,
      DisplayJoin(|| self.pats.iter().map(|x| display!(" {x}")), ""),
      self.body.display_pretty(2)
    )
  }
}

impl Term {
  pub fn display_pretty(&self, tab: usize) -> impl fmt::Display + '_ {
    maybe_grow(|| {
      DisplayFn(move |f| match self {
        Term::Lam { tag, nam, bod } => {
          write!(f, "{}λ{} {}", tag.display_padded(), var_as_str(nam), bod.display_pretty(tab))
        }

        Term::Var { nam } => write!(f, "{nam}"),

        Term::Chn { tag, nam, bod } => {
          write!(f, "{}λ${} {}", tag, var_as_str(nam), bod.display_pretty(tab))
        }

        Term::Lnk { nam } => write!(f, "${nam}"),

        Term::Let { nam, val, nxt } => {
          write!(
            f,
            "let {} = {};\n{:tab$}{}",
            var_as_str(nam),
            val.display_pretty(tab),
            "",
            nxt.display_pretty(tab)
          )
        }

        Term::Use { nam, val, nxt } => {
          write!(
            f,
            "use {} = {};\n{:tab$}{}",
            var_as_str(nam),
            val.display_pretty(tab),
            "",
            nxt.display_pretty(tab)
          )
        }

        Term::App { tag, fun, arg } => {
          write!(
            f,
            "{}({} {})",
            tag.display_padded(),
            fun.display_app_pretty(tag, tab),
            arg.display_pretty(tab)
          )
        }

        Term::Ltp { bnd, val, nxt } => {
          write!(
            f,
            "let ({}) = {};\n{:tab$}{}",
            DisplayJoin(|| bnd.iter().map(var_as_str), ", "),
            val.display_pretty(tab),
            "",
            nxt.display_pretty(tab),
          )
        }

        Term::Tup { els } => {
          write!(f, "({})", DisplayJoin(|| els.iter().map(|e| e.display_pretty(tab)), " "))
        }

        Term::Dup { tag, bnd, val, nxt } => {
          write!(
            f,
            "let {}{{{}}} = {};\n{:tab$}{}",
            tag.display_padded(),
            DisplayJoin(|| bnd.iter().map(var_as_str), " "),
            val.display_pretty(tab),
            "",
            nxt.display_pretty(tab),
          )
        }

        Term::Sup { tag, els } => {
          write!(
            f,
            "{}{{{}}}",
            tag.display_padded(),
            DisplayJoin(|| els.iter().map(|e| e.display_pretty(tab)), " ")
          )
        }

        Term::Lst { els } => {
          write!(f, "[{}]", DisplayJoin(|| els.iter().map(|e| e.display_pretty(tab)), " "))
        }

        Term::Opx { opr, fst, snd } => {
          write!(f, "({} {} {})", opr, fst.display_pretty(tab), snd.display_pretty(tab))
        }

        Term::Mat { bnd, arg, with, arms } => {
          let with: Box<dyn std::fmt::Display> = if with.is_empty() {
            Box::new(DisplayFn(|f| write!(f, "")))
          } else {
            Box::new(DisplayFn(|f| {
              write!(f, "with {}", DisplayJoin(|| with.iter().map(|e| e.to_string()), ", "))
            }))
          };
          writeln!(f, "match {} = {} {}{{", var_as_str(bnd), arg.display_pretty(tab), with)?;
          for rule in arms {
            writeln!(
              f,
              "{:tab$}{}: {};",
              "",
              var_as_str(&rule.0),
              rule.2.display_pretty(tab + 4),
              tab = tab + 2
            )?;
          }
          write!(f, "{:tab$}}}", "")?;
          Ok(())
        }

        Term::Swt { bnd, arg, with, pred: _, arms } => {
          let with: Box<dyn std::fmt::Display> = if with.is_empty() {
            Box::new(display!(""))
          } else {
            Box::new(display!(" with {}", DisplayJoin(|| with, ", ")))
          };
          writeln!(f, "switch {} = {} {}{{", var_as_str(bnd), arg.display_pretty(tab), with)?;
          for (i, rule) in arms.iter().enumerate() {
            writeln!(
              f,
              "{:tab$}{}: {};",
              "",
              if i == arms.len() - 1 { "_".to_string() } else { i.to_string() },
              rule.display_pretty(tab + 4),
              tab = tab + 2
            )?;
          }
          write!(f, "{:tab$}}}", "")?;
          Ok(())
        }

        Term::Nat { val } => write!(f, "#{val}"),
        Term::Num { val } => write!(f, "{val}"),
        Term::Str { val } => write!(f, "{val:?}"),
        Term::Ref { nam } => write!(f, "{nam}"),
        Term::Era => write!(f, "*"),
        Term::Err => write!(f, "<Error>"),
      })
    })
  }

  fn display_app_pretty<'a>(&'a self, tag: &'a Tag, tab: usize) -> impl fmt::Display + 'a {
    maybe_grow(|| {
      DisplayFn(move |f| match self {
        Term::App { tag: tag2, fun, arg } if tag2 == tag => {
          write!(f, "{} {}", fun.display_app_pretty(tag, tab), arg.display_pretty(tab))
        }
        _ => write!(f, "{}", self.display_pretty(tab)),
      })
    })
  }
}
