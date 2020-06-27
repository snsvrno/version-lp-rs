//! the main **version** struct.

use std::fmt;
use std::cmp::Ordering;

use serde;

use std::marker::PhantomData;

use crate::versionpart::VersionPart;

#[derive(Hash)]
pub struct Version {
    parts : Vec<VersionPart>
}

impl PartialEq for Version {
    fn eq(&self, other: &Version) -> bool {
        //! in order for a version to be equal all the parts need to be equal.
        //! and all parts need to be numbers `==` comparisons will always yield 
        //! false when comparing against a pattern.
        
        let depth : usize = Version::get_shared_depth(&self, other);

        for i in 0 .. depth {
            // checks if there is a wildcard, if there is then we assume the previous 
            // checks were all OK, and we ignore everything after a wildcard.
            if self.parts[i].is_wildcard() || other.parts[i].is_wildcard() { return true; }
            
            // if the two parts don't equal, and neither was a wildcard (above), then
            // we don't have the same version
            if self.parts[i] != other.parts[i] { return false}
        }

        // if we get to this point then they always matched, then we are the same
        return true;
    }
}

impl Eq for Version { }


impl std::cmp::Ord for Version {
    fn cmp(&self, other : &Version) -> Ordering {
        let depth : usize = Version::get_shared_depth(&self, other);

        // checks each parts, drilling down deeper in the version
        // struct
        for i in 0 .. depth {
            // checks if they are equal, if they are equal then 
            // we won't do anything and check the next part
            if self.parts[i] != other.parts[i] {
                // if they are not equal then we compare those parts
                // we only need to do this once and then return it
                return self.parts[i].cmp(&other.parts[i]);
            }
        }
        
        // we should never get here unless the two are the same ..
        Ordering::Equal
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other : &Version) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //! prints "Version(x.x.x)"
        write!(f, "Version ({})",self.to_string())
    }
}

impl fmt::Display for Version {
    fn fmt(&self,f:&mut fmt::Formatter) -> fmt::Result {
        //! prints "x.x.x"
        write!(f, "Version ({})",self.to_string())
    }
}


impl Version {

    /// checks how deep to compare. if they are different lenghts but all the 
    /// numbers of the same length (i.e. `"1.2.3.4" == "1.2.3"`) then it will assume
    /// that the smaller one has a wildcard at the end. 
    fn get_shared_depth(v1 : &Version, v2 : &Version) -> usize {

        if v1.parts.len() <= v2.parts.len() {
            return v1.parts.len() 
        } else { 
            return v2.parts.len(); 
        }
    }

    // initalizers

    /// creates a new version directly from an array of `u8`.
    pub fn new(numbers : &[u8]) -> Version {
        
        let mut parts : Vec<VersionPart> = Vec::new();

        for i in 0 .. numbers.len() {
            parts.push(VersionPart::Number(numbers[i]));
        }

        Version { parts : parts }
    }

    /// creates a new wildcard version,`*`, which matches compatible with everything
    pub fn new_wildcard() -> Version {

        Version { parts : vec!(VersionPart::Wildcard("*".to_string())) } 
    }

    /// creates a version from a string with a custom regex string.
    ///
    /// expecting a regex that returns unnamed capture groups and at most 3
    /// captures since the version string can only have 3 sections.
    //
    // the regex is automatically surrounded with `^` and `$` meaning that it will 
    // only match if it matches the entire string.
    pub fn from_str_with(version : &str, version_string_splitter : &str) -> Option<Version> {
        
        let mut parts : Vec<VersionPart> = Vec::new();

        for section in version.split(version_string_splitter) {
            match section.parse::<u8>() {
                Ok(number) => parts.push(VersionPart::Number(number)),
                Err(_) => {
                    // not a number so could be a wildcard??
                    if section == "*" {
                        parts.push(VersionPart::Wildcard(String::from(section)));
                        
                        // we ignore the rest of the string, so we just return this
                        return Some(Version { parts });
                    }
                    else {
                        // this isn't a version string then.
                        return None;
                    }
                }
            }
        }

        match parts.len() {
            0 => None,
            _ => Some(Version { parts })
        }

    }

    /// creates a version from a string
    pub fn from_str(version : &str) -> Option<Version> {
        Version::from_str_with(version, ".")
    }

    /// creates a disconnected copy
    pub fn clone(&self) -> Version { 
        Version::from_str(&self.to_string()).unwrap()
    }

    /// returns the largest version in the list of strings
    /// assumes they all aren't wildcards (doesn't process wildcards, just skips them from the list)
    ///
    /// if a string is passed that isn't a compatible version then it is ignored, no errors are made.
    pub fn from_latest_vec(list : &Vec<String>) -> Option<Version> {
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

    /// checks a list of strings which is the largest version number
    ///
    /// will parse the strings with `from_str` when comparing, if the string isn't valid
    /// it will skip it.
    pub fn latest_compatible<'a>(&self, list : &'a Vec<String>) -> Option<&'a str> {
        let mut latest = 0;
        for i in 1..list.len() {
            if let Some(ver) = Version::from_str(&list[i]){
                if ver.is_compatible_with(&self) { 
                    let ver_latest = Version::from_str(&list[latest]).unwrap();
                    if ver_latest < ver {
                        latest = i; 
                    }
                }
            }
        }

        if list.len() > 0 { Some(&list[latest]) } else { None } 
    }

    /// checks a list of versions and returns the one that is the largest compatible version
    ///
    /// uses implicit and explicit wildcards for the comparison.
    pub fn latest_compatible_version<'a>(&self,list : &'a Vec<Version>) -> Option<&'a Version> {
        let mut latest = 0;
        for i in 1..list.len() {
            if list[i].is_compatible_with(&self) { 
                if &list[latest] < &list[i] {
                    latest = i; 
                }
            }
        }

        if list.len() > 0 { Some(&list[latest]) } else { None } 
    }

    // checking functions, to get general booleans
    
    /// checks if the version has a wildcard in it
    pub fn has_wildcards(&self) -> bool { 
        
        for i in  0 .. self.parts.len() {
            if self.parts[i].is_wildcard() { return true; }
        }
        
        false
    }
    
    /// checks if the version is all numbers (no explicit wildcards)
    pub fn is_number(&self) -> bool { 
         
        for i in 0 .. self.parts.len() {
            if !self.parts[i].is_number() { return false; }
        }
        
        true
    }

    /// returns true if 100% wild (all defined sections are wildcards)
    pub fn is_wildcard(&self) -> bool {
         
        for i in 0 .. self.parts.len() {
            if self.parts[i].is_number() { return false; }
        }
        
        true
    }
    
    /// checks compatibility between versions
    ///
    /// uses wildcards in the comparision. if the `self` version has wildcards then it will not be 
    /// compatible with anything else since it is not an actual version
    pub fn is_compatible_with(&self,other : &Version) -> bool {
        // if the version number is a wildcard, it can not be compatible with anything else,
        // compatibility is only for compairing real numbers against other real or wildcard numbers
        if self.has_wildcards() { return false; }
        if other.is_wildcard() { return true; }

        // same version so it is compatible
        if self == other { return true; }

        let depth : usize = Version::get_shared_depth(&self, other);

        for i in 0 .. depth {
            if let VersionPart::Number(n) = self.parts[i] {
                match other.parts[i] {
                    VersionPart::Number(on) => { if on != n { return false; } },
                    VersionPart::Wildcard(_) => { return true; }
                }
            }
        }

        false
    }

    // data structure covnersion

    
    /// returns a string formated as "x.x.x.x"
    pub fn to_string(&self) -> String {
        
        let mut rendered_string : String = String::new();

        for i in 0 .. self.parts.len() - 1 {
            rendered_string += &format!("{}.",self.parts[i]); 
        }
        rendered_string += &format!("{}", self.parts[self.parts.len()-1]);

        return rendered_string;
    }

    /// returns a string formated as "x_x_x_x"
    pub fn to_string_serializer(&self) -> String {
        
        let mut rendered_string : String = String::new();

        for i in 0 .. self.parts.len() - 1 {
            rendered_string += &format!("{}_",self.parts[i]); 
        }
        rendered_string += &format!("{}", self.parts[self.parts.len()-1]);

        return rendered_string;
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

    fn visit_str<A>(self, string:&str) -> Result<Self::Value, A> 
    where A : serde::de::Error,
    {
        use serde::de::{Error, Unexpected};

        if let Some(version) = Version::from_str_with(string, "_") { 
            Ok(version) 
        } else { 
            Err(Error::invalid_value(Unexpected::Str(string), &self)) 
        }
    }
}


//////////////////////////////////////////////////////////////////////////////////////////////////////////////
// TESTS GO HERE

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparisons() {
        use super::Version as V;

        assert!(V::new(&[1,2,3]) == V::from_str("1.2.3").unwrap());
        assert!(V::new(&[1,2,3]) < V::new(&[2,3]));
        assert!(V::new(&[1,2,3]) <= V::new(&[1,2,4]));
        assert!(V::new(&[233]) > V::new(&[2,3]));
        assert!(V::new(&[22,3,56,8]) >= V::new(&[22,3,56]));

        // should it be less than since its technically not versioned as much?
        // this test will be for when the 'behavior change' is implemented
        // that adds strick version checking, no assumed wildcards.
        // assert_eq!(false,V::new(&[22,3,56]) >= V::new(&[22,3,56,223]));
    }

    #[test]
    fn sorting() {
        let mut vector = vec![
            super::Version::new(&[1,4,5]),
            super::Version::new(&[1,0,5]),
            super::Version::new(&[0,4,5]),
            super::Version::new(&[0,9,5]),
            super::Version::new(&[3,0,5])
        ];

        vector.sort();

        assert_eq!(vector[0],super::Version::new(&[0,4,5]));
        assert_eq!(vector[4],super::Version::new(&[3,0,5]));
        assert_eq!(vector[3],super::Version::new(&[1,4,5]));
    }

    #[test]
    fn version_is_compatible_with() {
        // various compatibility checks
        assert!(super::Version::from_str("0.1.0").unwrap().is_compatible_with(&super::Version::from_str("0.*.*").unwrap()));
        assert!(super::Version::from_str("4.1.0").unwrap().is_compatible_with(&super::Version::from_str("4.*.*").unwrap()));
        assert!(!super::Version::from_str("1.2.0").unwrap().is_compatible_with(&super::Version::from_str("1.1.*").unwrap()));
        assert!(!super::Version::from_str("11.1.*").unwrap().is_compatible_with(&super::Version::from_str("11.1.4").unwrap()));
        assert!(super::Version::from_str("1.1").unwrap().is_compatible_with(&super::Version::from_str("1.1.0").unwrap()));
        //assert!(!super::Version::from_str("2.1.0.2.43").unwrap().is_compatible_with(&super::Version::from_str("2.1.1").unwrap()));
        assert!(!super::Version::from_str("1.1.*").unwrap().is_compatible_with(&super::Version::from_str("1.*.*").unwrap()));
        assert!(super::Version::from_str("21.11.0").unwrap().is_compatible_with(&super::Version::from_str("*").unwrap()));
        // various failing ones
        assert_eq!(false,super::Version::from_str("21.11.0").unwrap().is_compatible_with(&super::Version::from_str("12.*").unwrap()));
        assert_eq!(false,super::Version::from_str("12.0").unwrap().is_compatible_with(&super::Version::from_str("12.1.2").unwrap()));
        assert_eq!(false,super::Version::from_str("21.11").unwrap().is_compatible_with(&super::Version::from_str("12").unwrap()));
        assert_eq!(false,super::Version::from_str("21.*").unwrap().is_compatible_with(&super::Version::from_str("22.12").unwrap()));
    }

    #[test]
    fn version_comparisons() {
        assert!(super::Version::from_str("1.1.0").unwrap() > super::Version::from_str("1.0.0").unwrap());
        assert!(super::Version::from_str("1.2.0").unwrap() < super::Version::from_str("1.3.1").unwrap());
        assert!(super::Version::from_str("2.3.2").unwrap() > super::Version::from_str("1.4.8").unwrap());
        assert!(super::Version::from_str("1.10.2").unwrap() > super::Version::from_str("1.4.22").unwrap());
    }

    #[test]
    fn latest_compatible() {
        let versions : Vec<String> = vec![
            "1.0.1".to_string(),
            "1.0.2".to_string(),
            "1.1.0".to_string(),
            "1.0.0".to_string()
        ];

        let version = Version::from_str("1.*.*").unwrap();

        assert_eq!(version.latest_compatible(&versions).unwrap().to_string(),"1.1.0".to_string());
    }

    #[test]
    fn latest_compatible_version() {
        let versions : Vec<Version> = vec![
            Version::from_str("1.0.0").unwrap(),
            Version::from_str("1.0.1").unwrap(),
            Version::from_str("1.0.2").unwrap(),
            Version::from_str("1.1.0").unwrap(),
            Version::from_str("2.3.132").unwrap()
        ];

        let version = Version::from_str("1.*.*").unwrap();
        assert_eq!(version.latest_compatible_version(&versions).unwrap().to_string(),"1.1.0".to_string());
        assert_eq!(Version::new(&[1]).latest_compatible_version(&versions).unwrap().to_string(),"1.1.0".to_string());
    }

    #[test]
    fn versionpart_is_number() {
        let vp = super::VersionPart::Number(12);
        let vp2 = super::VersionPart::Wildcard("*".to_string());
        assert!(vp.is_number());
        assert!(!vp2.is_number());
    }

    #[test]
    fn versionpart_is_wildcard() {
        let vp = super::VersionPart::Number(12);
        let vp2 = super::VersionPart::Wildcard("*".to_string());
        assert!(!vp.is_wildcard());
        assert!(vp2.is_wildcard());
    }

    #[test]
    fn versionpart_equals() {
        let vp = super::VersionPart::Number(14);
        let vp2 = super::VersionPart::Number(65);
        let vp3 = super::VersionPart::Wildcard("*".to_string());
        assert!(vp == super::VersionPart::Number(14));
        assert!(vp2 == super::VersionPart::Number(65));
        assert!(vp2 != vp);
        assert!(vp != vp3);
    }

    #[test]
    fn version_from_string() {
        let ver1 : super::Version = super::Version::from_str("1.0.0").unwrap();
        assert_eq!(super::Version::new(&[1,0,0]),ver1);

        let ver2 : super::Version = super::Version::from_str("1.1.0").unwrap();
        assert_eq!(super::Version::new(&[1,1,0]),ver2);
    }

    #[test]
    fn version_from_string_fails() {
        let ver1 = super::Version::from_str("x243");
        println!("{:?}",ver1);
        assert!(ver1.is_none());
    }

    #[test]
    fn serde() {
        use serde_test::{Token, assert_tokens};

        let version = super::Version::from_str("0.1.2").unwrap();
        assert_tokens(&version,&[Token::Str("0_1_2")]);

    }

    #[test]
    fn version_parse_serde() {
        let version = super::Version::from_str_with("0_1_2", "_").unwrap();
        assert_eq!(version, super::Version::new(&[0,1,2]));
    }

    #[test]
    fn basic_parsing() {
        assert_eq!(super::Version::from_str("0.1.2").unwrap(), super::Version::new(&[0,1,2]));
        assert_eq!(super::Version::from_str("120.1.2").unwrap(), super::Version::new(&[120,1,2]));
        assert_eq!(super::Version::from_str("1.12.2").unwrap(), super::Version::new(&[1,12,2]));
        assert_eq!(super::Version::from_str("1.1.132").unwrap(), super::Version::new(&[1,1,132]));
        assert_eq!(super::Version::from_str("0.0.2").unwrap(), super::Version::new(&[0,0,2]));
        assert_eq!(super::Version::from_str("0132.1.2").unwrap(), super::Version::new(&[132,1,2]));
        assert_eq!(super::Version::from_str("1.2.3.12.123.231.111").unwrap(), super::Version::new(&[1,2,3,12,123,231,111]));
        assert_eq!(super::Version::from_str("1.2").unwrap(), super::Version::new(&[1,2]));
        assert_eq!(super::Version::from_str("1").unwrap(), super::Version::new(&[1]));
    }


}