//! the main **version** struct.

use std::fmt;
use std::cmp::Ordering;

use regex::Regex;
use serde;

use std::marker::PhantomData;

use versionpart::VersionPart;

static VERSION_REGEX_STRING : &str = r"([1234567890\*]+)[.|-|_]([1234567890\*]+)[.|-|_]([1234567890\*]+)";

#[derive(Hash)]
pub struct Version {
  major : VersionPart,
  minor : VersionPart,
  patch : VersionPart
}

impl PartialEq for Version {
  fn eq(&self, other: &Version) -> bool {
    //! in order for a version to be equal all the parts need to be equal. `==` comparisons will always yield fals when comparing against a pattern.
    self.major == other.major && self.minor == other.minor && self.patch == other.patch
  }
}

impl Eq for Version { }

impl PartialOrd for Version {
  fn partial_cmp(&self, other : &Version) -> Option<Ordering> {
    if self.major != other.major { return Some(self.major.cmp(&other.major)); }
    else {
      if self.minor != other.minor { return Some(self.minor.cmp(&other.minor)); }
      else {
        return Some(self.patch.cmp(&other.patch));
      }
    }
  }
}

impl fmt::Debug for Version {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    //! prints "Version(major.minor.patch)"
    write!(f, "Version ({}.{}.{})",self.major,self.minor,self.patch)
  }
}

impl fmt::Display for Version {
  fn fmt(&self,f:&mut fmt::Formatter) -> fmt::Result {
    //! prints "major.minor.patch"
    write!(f,"{}.{}.{}",self.major,self.minor,self.patch)
  }
}

impl Version {

  // initalizers
  pub fn new(major:u8,minor:u8,patch:u8) -> Version {
    //! creates a new version directly.
    Version { major: VersionPart::Number(major), minor: VersionPart::Number(minor), patch: VersionPart::Number(patch) } 
  }

  pub fn new_wildcard() -> Version {
    //! creates a new wildcard version,"*", which matches compatible with everything
    Version { major: VersionPart::Wildcard("*".to_string()), minor: VersionPart::Wildcard("*".to_string()), patch: VersionPart::Wildcard("*".to_string()) } 
  }

  pub fn from_str(version : &str) -> Option<Version> {
    //! creates a version from a string

    // checks if it is a total wildcard version
    if version == "*" { return Some(Version::new_wildcard()); }

    let re = Regex::new(VERSION_REGEX_STRING).unwrap();
    if let Some(captures) = re.captures(version) {

      let mut opt_major : Option<VersionPart> = None;
      let mut opt_minor : Option<VersionPart> = None;
      let mut opt_patch : Option<VersionPart> = None;

      let major = captures.get(1).unwrap().as_str();
      match major.parse::<u8>() {
        Err(_) => { opt_major = Some(VersionPart::Wildcard(major.to_string())); }
        Ok(num) => { opt_major = Some(VersionPart::Number(num)); }
      }
      let minor = captures.get(2).unwrap().as_str();
      match minor.parse::<u8>() {
        Err(_) => { opt_minor = Some(VersionPart::Wildcard(minor.to_string())); }
        Ok(num) => { opt_minor = Some(VersionPart::Number(num)); }
      }
      let patch = captures.get(3).unwrap().as_str();
      match patch.parse::<u8>() {
        Err(_) => { opt_patch = Some(VersionPart::Wildcard(patch.to_string())); }
        Ok(num) => { opt_patch = Some(VersionPart::Number(num)); }
      }

      if opt_major.is_some() && opt_minor.is_some() && opt_patch.is_some() { 
        return Some(Version { major: opt_major.unwrap(), minor: opt_minor.unwrap(), patch: opt_patch.unwrap() });
      } else { return None; }

    }
    None
  }

  pub fn clone(&self) -> Version { 
    //! creates a disconnected copy
    Version::from_str(&self.to_string()).unwrap()
  }

  pub fn from_latest_vec(list : &Vec<String>) -> Option<Version> {
    //! returns the largest number in the list of strings
    //! assumes they all aren't wildcards (doesn't process wildcards, just skips them from the list)

    let mut list_of_versions : Vec<Version> = Vec::new();
    let mut selected = 0;

    for l in list { 
      if let Some(ver) = Version::from_str(l) { 
        if !ver.has_wildcards() { 
          list_of_versions.push(ver); 
        }
      } 
    }

    if list_of_versions.len() <= 0 { return None; }

    for cc in 1..list_of_versions.len() { 
      if list_of_versions[cc] > list_of_versions[selected] { selected = cc; } 
    }

    Some(list_of_versions.remove(selected))
  }

  // checking functions, to get general booleans
  pub fn has_wildcards(&self) -> bool { 
    //! checks if the version has a wildcard in it
    self.major.is_wildcard() || self.minor.is_wildcard() || self.patch.is_wildcard()
  }
  pub fn is_number(&self) -> bool { 
    //! checks if the version is all numbers
    self.major.is_number() && self.minor.is_number() && self.patch.is_number()
  }
  
  pub fn is_compatible_with(&self,other : &Version) -> bool {
    //! checks compatibility between versions
    //!
    //! uses wildcards in the comparision. if the `self` version has wildcards then it will not be compatible with anything else since it is not an actual version

    // if the version number is a wildcard, it can not be compatible with anything else,
    // compatibility is only for compairing real numbers against other real or wildcard numbers
    if self.has_wildcards() { return false; }

    // same version so it is compatible
    if self == other { return true; }

    if let VersionPart::Number(s1) = self.major {
      if let VersionPart::Number(s2) = self.minor {
        if let VersionPart::Number(s3) = self.patch {

          // since the only wildcard now is * only check for strings
          if let VersionPart::Number(o) = other.major { if o != s1 { return false; } }
          if let VersionPart::Number(o) = other.minor { if o != s2 { return false; } }
          if let VersionPart::Number(o) = other.patch { if o != s3 { return false; } }

          return true;
        }
      }
    }
    false
    //let first_match = if self.major == version.major { true } else if { self.major == "*"}
  }

  // data structure covnersion
  pub fn to_string(&self) -> String { 
    //! returns a string formated as "major.minor.patch"
    format!("{}.{}.{}",self.major,self.minor,self.patch)
  }

  pub fn to_string_serializer(&self) -> String {
    format!("{}_{}_{}",self.major,self.minor,self.patch)
  }
}

impl serde::Serialize for Version {
  fn serialize<S>(&self,serializer : S) -> Result<S::Ok, S::Error> where S : serde::Serializer {
    serializer.serialize_str(&self.to_string_serializer())
  }
}

impl <'de> serde::Deserialize<'de> for Version {
  fn deserialize<D>(deserializer : D) -> Result<Version, D::Error> where D : serde::Deserializer<'de> {
    deserializer.deserialize_str(VersionVisitor::new())
  }
}

struct VersionVisitor {
  marker: PhantomData<fn() -> Version>
}

impl VersionVisitor {
  fn new() -> Self {
    VersionVisitor {
      marker : PhantomData
    }
  }
}

impl <'de>serde::de::Visitor<'de> for VersionVisitor {
  type Value = Version;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("version")
  }

  fn visit_str<A>(self, string:&str) -> Result<Self::Value, A> {
    if let Some(version) = Version::from_str(string) { Ok(version) } 
    else { Ok(Version::from_str("0.0.0").unwrap()) }
  }
}


//////////////////////////////////////////////////////////////////////////////////////////////////////////////
// TESTS GO HERE

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn version_is_compatible_with() {
    assert_eq!(super::Version::from_str("0.1.0").unwrap().is_compatible_with(&super::Version::from_str("0.*.*").unwrap()),true);
    assert_eq!(super::Version::from_str("4.1.0").unwrap().is_compatible_with(&super::Version::from_str("4.*.*").unwrap()),true);
    assert_eq!(super::Version::from_str("1.2.0").unwrap().is_compatible_with(&super::Version::from_str("1.1.*").unwrap()),false);
    assert_eq!(super::Version::from_str("11.1.*").unwrap().is_compatible_with(&super::Version::from_str("11.1.4").unwrap()),false);
    assert_eq!(super::Version::from_str("1.1.0").unwrap().is_compatible_with(&super::Version::from_str("1.1.0").unwrap()),true);
    assert_eq!(super::Version::from_str("1233.11.0").unwrap().is_compatible_with(&super::Version::from_str("*").unwrap()),true);
    assert_eq!(super::Version::from_str("2.1.0").unwrap().is_compatible_with(&super::Version::from_str("2.1.1").unwrap()),false);
    assert_eq!(super::Version::from_str("1.1.*").unwrap().is_compatible_with(&super::Version::from_str("1.*.*").unwrap()),false);
  }

  #[test]
  fn versionpart_is_number() {
    let vp = super::VersionPart::Number(12);
    let vp2 = super::VersionPart::Wildcard("*".to_string());
    assert_eq!(vp.is_number(),true);
    assert_eq!(vp2.is_number(),false);
  }

  #[test]
  fn versionpart_is_wildcard() {
    let vp = super::VersionPart::Number(12);
    let vp2 = super::VersionPart::Wildcard("*".to_string());
    assert_eq!(vp.is_wildcard(),false);
    assert_eq!(vp2.is_wildcard(),true);
  }

  #[test]
  fn versionpart_equals() {
    let vp = super::VersionPart::Number(14);
    let vp2 = super::VersionPart::Number(65);
    let vp3 = super::VersionPart::Wildcard("*".to_string());
    assert_eq!(vp == super::VersionPart::Number(14),true);
    assert_eq!(vp2 == super::VersionPart::Number(65),true);
    assert_eq!(vp2 == vp,false);
    assert_eq!(vp == vp3,false);
  }

  #[test]
  fn version_from_string() {
    let ver1 : super::Version = super::Version::from_str("1.0.0").unwrap();
    assert_eq!(super::Version::new(1,0,0),ver1);
  }

  #[test]
  fn serde() {
    use serde_test::{Token, assert_tokens};

    let version = super::Version::from_str("0.1.2").unwrap();
    assert_tokens(&version,&[Token::Str("0_1_2")]);

  }


}