//! the major, minor, patch sections of the version

use std::fmt;
use std::cmp::Ordering;

#[derive(Hash)]
pub enum VersionPart {
  Number(u8),
  Wildcard(String)
}

impl VersionPart { 
  pub fn is_number(&self) -> bool {
    match self {
      &VersionPart::Number(_) => { return true; }
      &VersionPart::Wildcard(_) => { return false; }
    }
  }
  pub fn is_wildcard(&self) -> bool {
    match self {
      &VersionPart::Number(_) => { return false; }
      &VersionPart::Wildcard(_) => { return true; }
    }
  }
}

impl PartialEq for VersionPart {
  fn eq(&self, other: &VersionPart) -> bool {
    //! equals is only for numbers, not pattern matching

    if let &VersionPart::Number(a) = self {
      if let &VersionPart::Number(b) = other {
      return a == b;
      }
    } 

    false
  }
}

impl Eq for VersionPart { }

impl Ord for VersionPart {
  fn cmp(&self, other :&VersionPart) -> Ordering {
    //! a wildcard is always the greatest possible number when sorting
    
    if self.is_wildcard() && other.is_wildcard() { return Ordering::Equal; }
    if self.is_wildcard() && other.is_number() { return Ordering::Greater; }
    if self.is_number() && other.is_wildcard() { return Ordering::Less; }
    
    if let &VersionPart::Number(ref s) = self { 
      if let &VersionPart::Number(ref o) = other {
        return s.cmp(o); 
      }
    }

    Ordering::Equal // should never return this, but don't know how to do the number.cmp.number correctly
  }
}

impl PartialOrd for VersionPart {
  fn partial_cmp(&self, other :&VersionPart) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl fmt::Display for VersionPart {
  fn fmt(&self,f:&mut fmt::Formatter) -> fmt::Result {
    match self {
      &VersionPart::Number(ref num) => { write!(f,"{}",num) }
      &VersionPart::Wildcard(ref string) => { write!(f,"{}",string) }
    }
  }
}
