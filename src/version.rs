//! the main **version** struct.

use std::fmt;
use std::cmp::Ordering;

use regex::Regex;
use serde;

use std::marker::PhantomData;

use crate::versionpart::VersionPart;

static VERSION_REGEX_STRING : &str = r"(?P<v>[1234567890\*]+)";

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

    fn get_shared_depth(v1 : &Version, v2 : &Version) -> usize {
    //! checks how deep to compare. if they are different lenghts but all the 
    //! numbers of the same length (i.e. `"1.2.3.4" == "1.2.3"`) then it will assume
    //! that the smaller one has a wildcard at the end. 
        if v1.parts.len() <= v2.parts.len() {
            return v1.parts.len() 
        } else { 
            return v2.parts.len(); 
        }
    }

    // initalizers
    pub fn new(numbers : &[u8]) -> Version {
        //! creates a new version directly.
        
        let mut parts : Vec<VersionPart> = Vec::new();

        for i in 0 .. numbers.len() {
            parts.push(VersionPart::Number(numbers[i]));
        }

        Version { parts : parts }
    }

    pub fn new_wildcard() -> Version {
        //! creates a new wildcard version,"*", which matches compatible with everything

        Version { parts : vec!(VersionPart::Wildcard("*".to_string())) } 
    }

    pub fn from_str(version : &str) -> Option<Version> {
        //! creates a version from a string

        let re = Regex::new(VERSION_REGEX_STRING).unwrap();
        let mut parts : Vec<VersionPart> = Vec::new();

        for caps in re.captures_iter(version) {
            // there should be only one capture per match, but there could be multiple matches
            match caps["v"].parse::<u8>() {
                Err(_) => {
                    // its not a number, so if it passes the regex part it must be a wildcard
                    parts.push(VersionPart::Wildcard(caps["v"].to_string()));

                    // if it matches a wild card it will ignore everything afterwards.
                    return Some(Version{
                        parts : parts
                    });
                },
                Ok(number) => {
                    // is a number, so an actual version number
                    parts.push(VersionPart::Number(number));
                }
            }
        }

        if parts.len() > 0 { 
            Some(Version{ 
                parts : parts 
            }) 
        } else { 
            None 
        }
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

    pub fn latest_compatible<'a>(&self,list : &'a Vec<String>) -> Option<&'a str> {
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
    pub fn has_wildcards(&self) -> bool { 
        //! checks if the version has a wildcard in it
        
        for i in  0 .. self.parts.len() {
            if self.parts[i].is_wildcard() { return true; }
        }
        
        false
    }
    pub fn is_number(&self) -> bool { 
        //! checks if the version is all numbers
         
        for i in 0 .. self.parts.len() {
            if !self.parts[i].is_number() { return false; }
        }
        
        true
    }

    pub fn is_wildcard(&self) -> bool {
        //! returns true if 100% wild
         
        for i in 0 .. self.parts.len() {
            if self.parts[i].is_number() { return false; }
        }
        
        true
    }
    
    pub fn is_compatible_with(&self,other : &Version) -> bool {
        //! checks compatibility between versions
        //!
        //! uses wildcards in the comparision. if the `self` version has wildcards then it will not be compatible with anything else since it is not an actual version

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
    pub fn to_string(&self) -> String {
        //! returns a string formated as "x.x.x.x"
        
        let mut rendered_string : String = String::new();

        for i in 0 .. self.parts.len() - 1 {
            rendered_string += &format!("{}.",self.parts[i]); 
        }
        rendered_string += &format!("{}", self.parts[self.parts.len()-1]);

        return rendered_string;
    }

    pub fn to_string_serializer(&self) -> String {
        //! returns a string formated as "x_x_x_x"
        
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
        assert!(!super::Version::from_str("2.1.0.2.43").unwrap().is_compatible_with(&super::Version::from_str("2.1.1").unwrap()));
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
            Version::from_str("1.1.0").unwrap()
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
    }

    #[test]
    fn serde() {
        use serde_test::{Token, assert_tokens};

        let version = super::Version::from_str("0.1.2").unwrap();
        assert_tokens(&version,&[Token::Str("0_1_2")]);

    }


}