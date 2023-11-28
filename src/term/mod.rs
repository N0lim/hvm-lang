use hvmc::run::Val;
use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
use shrinkwraprs::Shrinkwrap;
use std::{
  collections::{BTreeMap, HashMap},
  fmt,
};

pub mod check;
pub mod load_book;
pub mod net_to_term;
pub mod parser;
pub mod term_to_net;
pub mod transform;

pub use term_to_net::{book_to_nets, term_to_compat_net};

/// The representation of a program.
#[derive(Debug, Clone, Default)]
pub struct Book {
  /// Mapping of definition names to ids.
  pub def_names: DefNames,

  /// The function definitions.
  pub defs: BTreeMap<DefId, Definition>,

  /// The algebraic datatypes defined by the program
  pub adts: BTreeMap<Name, Adt>,

  /// To which type does each constructor belong to.
  pub ctrs: HashMap<Name, Name>,
}

#[derive(Debug, Clone, Default)]
pub struct DefNames {
  id_to_name: HashMap<DefId, Name>,
  name_to_id: HashMap<Name, DefId>,
  id_count: DefId,
}

/// A pattern matching function definition.
#[derive(Debug, Clone)]
pub struct Definition {
  pub def_id: DefId,
  pub rules: Vec<Rule>,
}

/// A pattern matching rule of a definition.
#[derive(Debug, Clone)]
pub struct Rule {
  pub pats: Vec<RulePat>,
  pub body: Term,
}

#[derive(Debug, Clone)]
pub enum MatchNum {
  Zero,
  Succ(Option<Name>),
}

#[derive(Debug, Clone)]
pub enum RulePat {
  Var(Name),
  Ctr(Name, Vec<RulePat>),
  Num(MatchNum),
}

#[derive(Debug, Clone, Default)]
pub enum Term {
  Lam {
    tag: Option<Name>,
    nam: Option<Name>,
    bod: Box<Term>,
  },
  Var {
    nam: Name,
  },
  /// Like a scopeless lambda, where the variable can occur outside the body
  Chn {
    nam: Name,
    bod: Box<Term>,
  },
  /// The use of a Channel variable.
  Lnk {
    nam: Name,
  },
  Let {
    pat: LetPat,
    val: Box<Term>,
    nxt: Box<Term>,
  },
  App {
    tag: Option<Name>,
    fun: Box<Term>,
    arg: Box<Term>,
  },
  Tup {
    fst: Box<Term>,
    snd: Box<Term>,
  },
  Dup {
    tag: Option<Name>,
    fst: Option<Name>,
    snd: Option<Name>,
    val: Box<Term>,
    nxt: Box<Term>,
  },
  Sup {
    tag: Name,
    fst: Box<Term>,
    snd: Box<Term>,
  },
  Num {
    val: u32,
  },
  /// A numeric operation between built-in numbers.
  Opx {
    op: Op,
    fst: Box<Term>,
    snd: Box<Term>,
  },
  Match {
    scrutinee: Box<Term>,
    arms: Vec<(RulePat, Term)>,
  },
  Ref {
    def_id: DefId,
  },
  #[default]
  Era,
}

#[derive(Debug, Clone)]
pub enum LetPat {
  Var(Name),
  Tup(Option<Name>, Option<Name>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
  ADD,
  SUB,
  MUL,
  DIV,
  MOD,
  EQ,
  NE,
  LT,
  GT,
  AND,
  OR,
  XOR,
  NOT,
  LSH,
  RSH,
}

/// A user defined  datatype
#[derive(Debug, Clone, Default)]
pub struct Adt {
  pub ctrs: IndexMap<Name, Vec<Name>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Shrinkwrap, Hash, PartialOrd, Ord)]
pub struct Name(pub String);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Shrinkwrap, Hash, PartialOrd, Ord, Default)]
pub struct DefId(pub Val);

pub fn var_id_to_name(mut var_id: Val) -> Name {
  let mut name = String::new();
  loop {
    let c = (var_id % 26) as u8 + b'a';
    name.push(c as char);
    var_id /= 26;
    if var_id == 0 {
      break;
    }
  }
  Name(name)
}

impl Name {
  pub fn new(value: &str) -> Self {
    Name(value.to_string())
  }
}

impl DefId {
  // TODO: We use this workaround because hvm-core's val_to_name function doesn't work with value 0
  pub fn to_internal(self) -> Val {
    *self + 1
  }

  pub fn from_internal(val: Val) -> Self {
    Self(val - 1)
  }
}

impl Book {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn insert_def(&mut self, name: Name, rules: Vec<Rule>) -> DefId {
    let def_id = self.def_names.insert(name);
    let def = Definition { def_id, rules };
    self.defs.insert(def_id, def);
    def_id
  }

  pub fn remove_def(&mut self, def_id: DefId) -> Option<(Name, Definition)> {
    let def = self.defs.remove(&def_id);
    let name = self.def_names.remove(def_id);
    name.zip(def)
  }
}

impl DefNames {
  pub const ENTRY_POINT: &'static str = "main";
  pub const HVM1_ENTRY_POINT: &'static str = "Main";

  pub fn new() -> Self {
    Default::default()
  }

  pub fn name(&self, def_id: &DefId) -> Option<&Name> {
    self.id_to_name.get(def_id)
  }

  pub fn def_id(&self, name: &Name) -> Option<DefId> {
    self.name_to_id.get(name).copied()
  }

  pub fn contains_name(&self, name: &Name) -> bool {
    self.name_to_id.contains_key(name)
  }

  pub fn contains_def_id(&self, def_id: &DefId) -> bool {
    self.id_to_name.contains_key(def_id)
  }

  pub fn insert(&mut self, name: Name) -> DefId {
    let def_id = self.id_count;
    self.id_count.0 += 1;
    self.id_to_name.insert(def_id, name.clone());
    self.name_to_id.insert(name, def_id);
    def_id
  }

  pub fn remove(&mut self, def_id: DefId) -> Option<Name> {
    let nam = self.id_to_name.remove(&def_id);
    if let Some(nam) = &nam {
      self.name_to_id.remove(nam);
    }
    nam
  }

  pub fn names(&self) -> impl Iterator<Item = &Name> {
    self.name_to_id.keys()
  }

  pub fn def_ids(&self) -> impl Iterator<Item = &DefId> {
    self.id_to_name.keys()
  }
}

impl Term {
  fn to_string_app_head(&self, book: &Book) -> String {
    match self {
      Term::App { tag: None, fun, arg } => {
        format!("{} {}", fun.to_string_app_head(book), arg.to_string(book))
      }
      _ => self.to_string(book),
    }
  }
  pub fn to_string(&self, book: &Book) -> String {
    match self {
      Term::Lam { tag, nam, bod, .. } => {
        format!(
          "λ{}{} {}",
          match tag {
            Some(x) => format!("#{} ", x),
            _ => "".to_owned(),
          },
          nam.clone().unwrap_or(Name::new("*")),
          bod.to_string(book)
        )
      }
      Term::Var { nam } => format!("{nam}"),
      Term::Chn { nam, bod } => format!("λ${} {}", nam, bod.to_string(book)),
      Term::Lnk { nam } => format!("${nam}"),
      Term::Let { pat, val, nxt } => {
        format!("let {} = {}; {}", pat, val.to_string(book), nxt.to_string(book))
      }
      Term::Ref { def_id } => format!("{}", book.def_names.name(def_id).unwrap()),
      Term::App { tag: None, fun, arg } => {
        format!("({} {})", fun.to_string_app_head(book), arg.to_string(book))
      }
      Term::App { tag, fun, arg } => {
        format!(
          "({}{} {})",
          match tag {
            Some(x) => format!("#{} ", x),
            _ => "".to_owned(),
          },
          fun.to_string_app_head(book),
          arg.to_string(book)
        )
      }
      Term::Match { scrutinee, arms } => {
        let arms = arms.iter().map(|(pat, term)| format!("{}: {}", pat, term.to_string(book))).join("; ");

        format!("match {} {{ {} }}", scrutinee.to_string(book), arms,)
      }
      Term::Dup { tag: _, fst, snd, val, nxt } => format!(
        "dup {} {} = {}; {}",
        fst.as_ref().map(|x| x.as_str()).unwrap_or("*"),
        snd.as_ref().map(|x| x.as_str()).unwrap_or("*"),
        val.to_string(book),
        nxt.to_string(book)
      ),
      Term::Sup { tag, fst, snd } => {
        format!("{{#{} {} {}}}", tag, fst.to_string(book), snd.to_string(book))
      }
      Term::Era => "*".to_string(),
      Term::Num { val } => format!("{val}"),
      Term::Opx { op, fst, snd } => {
        format!("({} {} {})", op, fst.to_string(book), snd.to_string(book))
      }
      Term::Tup { fst, snd } => format!("({}, {})", fst.to_string(book), snd.to_string(book)),
    }
  }

  /// Make a call term by folding args around a called function term with applications.
  pub fn call(called: Term, args: impl IntoIterator<Item = Term>) -> Self {
    args.into_iter().fold(called, |acc, arg| Term::App { tag: None, fun: Box::new(acc), arg: Box::new(arg) })
  }

  /// Substitute the occurences of a variable in a term with the given term.
  pub fn subst(&mut self, from: &Name, to: &Term) {
    match self {
      Term::Lam { nam: Some(nam), .. } if nam == from => (),
      Term::Lam { bod, .. } => bod.subst(from, to),
      Term::Var { nam } if nam == from => *self = to.clone(),
      Term::Var { .. } => (),
      // Only substitute scoped variables.
      Term::Chn { bod, .. } => bod.subst(from, to),
      Term::Lnk { .. } => (),
      Term::Let { pat: LetPat::Var(nam), val, nxt } => {
        val.subst(from, to);
        if nam != from {
          nxt.subst(from, to);
        }
      }
      Term::Let { pat: LetPat::Tup(fst, snd), val, nxt } => {
        val.subst(from, to);
        if fst.as_ref().map_or(true, |fst| fst != from) && snd.as_ref().map_or(true, |snd| snd != from) {
          nxt.subst(from, to);
        }
      }
      Term::Dup { tag: _, fst, snd, val, nxt } => {
        val.subst(from, to);
        if fst.as_ref().map_or(true, |fst| fst != from) && snd.as_ref().map_or(true, |snd| snd != from) {
          nxt.subst(from, to);
        }
      }
      Term::Match { scrutinee, arms } => {
        scrutinee.subst(from, to);

        for (rule, term) in arms {
          let can_subst;

          if let RulePat::Num(MatchNum::Succ(Some(nam))) = rule {
            can_subst = nam != from
          } else {
            can_subst = true
          };

          if can_subst {
            term.subst(from, to);
          }
        }
      }
      Term::App { fun: fst, arg: snd, .. }
      | Term::Sup { fst, snd, .. }
      | Term::Tup { fst, snd }
      | Term::Opx { fst, snd, .. } => {
        fst.subst(from, to);
        snd.subst(from, to);
      }
      Term::Ref { .. } | Term::Num { .. } | Term::Era => (),
    }
  }

  /// Collects all the free variables that a term has
  pub fn free_vars(&self, free_vars: &mut IndexSet<Name>) {
    match self {
      Term::Lam { nam: Some(nam), bod, .. } => {
        let mut new_scope = IndexSet::new();
        bod.free_vars(&mut new_scope);
        new_scope.remove(nam);

        free_vars.extend(new_scope);
      }
      Term::Lam { nam: None, bod, .. } => bod.free_vars(free_vars),
      Term::Var { nam } => _ = free_vars.insert(nam.clone()),
      Term::Chn { bod, .. } => bod.free_vars(free_vars),
      Term::Lnk { .. } => {}
      Term::Let { pat: LetPat::Var(nam), val, nxt } => {
        val.free_vars(free_vars);

        let mut new_scope = IndexSet::new();
        nxt.free_vars(&mut new_scope);

        new_scope.remove(nam);

        free_vars.extend(new_scope);
      }
      Term::Let { pat: LetPat::Tup(fst, snd), val, nxt } | Term::Dup { fst, snd, val, nxt, .. } => {
        val.free_vars(free_vars);

        let mut new_scope = IndexSet::new();
        nxt.free_vars(&mut new_scope);

        fst.as_ref().map(|fst| new_scope.remove(fst));
        snd.as_ref().map(|snd| new_scope.remove(snd));

        free_vars.extend(new_scope);
      }
      Term::App { fun: fst, arg: snd, .. }
      | Term::Tup { fst, snd }
      | Term::Sup { fst, snd, .. }
      | Term::Opx { op: _, fst, snd } => {
        fst.free_vars(free_vars);
        snd.free_vars(free_vars);
      }
      Term::Match { scrutinee, arms } => {
        scrutinee.free_vars(free_vars);

        for (rule, term) in arms {
          let mut new_scope = IndexSet::new();
          term.free_vars(&mut new_scope);

          if let RulePat::Num(MatchNum::Succ(Some(nam))) = rule {
            new_scope.remove(nam);
          }

          free_vars.extend(new_scope);
        }
      }
      Term::Ref { .. } | Term::Num { .. } | Term::Era => {}
    }
  }

  /// Creates a new [`Term::Match`] from the given terms.
  /// If `scrutinee` is not a `Term::Var`, creates a let binding containing the match in its body
  pub fn new_native_match(
    scrutinee: Self,
    zero_term: Self,
    succ_label: Option<Name>,
    succ_term: Self,
  ) -> Self {
    let zero = (RulePat::Num(MatchNum::Zero), zero_term);

    let succ = (RulePat::Num(MatchNum::Succ(succ_label)), succ_term);
    Term::Match { scrutinee: Box::new(scrutinee), arms: vec![zero, succ] }
  }
}

/// Returns the lambda representation of native number match arms
pub fn native_match(arms: Vec<(RulePat, Term)>) -> (Term, Term) {
  use MatchNum::*;

  match &arms[..] {
    [(RulePat::Num(Zero), zero), (RulePat::Num(Succ(nam)), succ)] => {
      let zero = zero.clone();
      let succ = Term::Lam { tag: None, nam: nam.clone(), bod: Box::new(succ.clone()) };
      (zero, succ)
    }
    [(RulePat::Num(Zero), zero), (RulePat::Num(Zero), succ)] => (zero.clone(), succ.clone()),
    _ => unreachable!(),
  }
}

impl fmt::Display for LetPat {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      LetPat::Var(nam) => write!(f, "{}", nam),
      LetPat::Tup(fst, snd) => {
        write!(
          f,
          "({}, {})",
          fst.as_ref().map(|s| s.to_string()).unwrap_or("*".to_string()),
          snd.as_ref().map(|s| s.to_string()).unwrap_or("*".to_string()),
        )
      }
    }
  }
}

impl Rule {
  pub fn to_string(&self, def_id: &DefId, book: &Book) -> String {
    format!(
      "({}{}) = {}",
      book.def_names.name(def_id).unwrap(),
      self.pats.iter().map(|x| format!(" {x}")).join(""),
      self.body.to_string(book)
    )
  }

  pub fn arity(&self) -> usize {
    self.pats.len()
  }
}

impl Definition {
  pub fn to_string(&self, book: &Book) -> String {
    self.rules.iter().map(|x| x.to_string(&self.def_id, book)).join("\n")
  }

  pub fn arity(&self) -> usize {
    self.rules[0].arity()
  }

  pub fn assert_no_pattern_matching_rules(&self) {
    assert!(self.rules.len() == 1, "Definition rules should have been removed in earlier pass");
  }
}

impl fmt::Display for Book {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.defs.values().map(|x| x.to_string(&self)).join("\n\n"))
  }
}

impl From<&RulePat> for Term {
  fn from(value: &RulePat) -> Self {
    match value {
      RulePat::Ctr(nam, args) => Term::call(Term::Var { nam: nam.clone() }, args.iter().map(Term::from)),
      RulePat::Var(nam) => Term::Var { nam: nam.clone() },
      RulePat::Num(..) => todo!(),
    }
  }
}

impl fmt::Display for RulePat {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      RulePat::Ctr(name, pats) => write!(f, "({}{})", name, pats.iter().map(|p| format!(" {p}")).join("")),
      RulePat::Var(nam) => write!(f, "{}", nam),
      RulePat::Num(MatchNum::Zero) => write!(f, "0"),
      RulePat::Num(MatchNum::Succ(_)) => write!(f, "+"),
    }
  }
}

impl fmt::Display for Op {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Op::ADD => write!(f, "+"),
      Op::SUB => write!(f, "-"),
      Op::MUL => write!(f, "*"),
      Op::DIV => write!(f, "/"),
      Op::MOD => write!(f, "%"),
      Op::EQ => write!(f, "=="),
      Op::NE => write!(f, "!="),
      Op::LT => write!(f, "<"),
      Op::GT => write!(f, ">"),
      Op::AND => write!(f, "&"),
      Op::OR => write!(f, "|"),
      Op::XOR => write!(f, "^"),
      Op::NOT => write!(f, "~"),
      Op::LSH => write!(f, "<<"),
      Op::RSH => write!(f, ">>"),
    }
  }
}

impl fmt::Display for Name {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}
