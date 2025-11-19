use std::fmt;


#[derive(Debug, PartialEq, Clone)]
pub struct SaveNameInfo {
    pub tag: String,
    pub version: Option<String>,
    pub backup_id: Option<String>,
    pub internal_tag: Option<String>,
}

impl SaveNameInfo {
  pub fn new(tag: &str, version: Option<&str>, backup: Option<&str>, internal_tag: Option<&str>) -> Self {
    SaveNameInfo { 
      tag: tag.to_owned(), 
      version: version.map(|x| x.to_owned()), 
      backup_id: backup.map(|x| x.to_owned()), 
      internal_tag: internal_tag.map(|x| x.to_owned())
    }
  }
}

impl fmt::Display for SaveNameInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        // Version tag, is has some, add a prefix '_' before version tag.
        //
        // Example:
        //  - With version tag: user2_1.0.29242.dat 
        //  - Without version tag: user2.dat 
        let ver_str = 
            &self.version.clone().map_or("".to_owned(), 
            |x: String| format!("_{}", x)
            
        );

        // Suffix and extension tag, if have some backup tag, add special
        // extension for backups. Backups can be empty string.
        //
        // Example:
        //  - With backup tag: user3.dat.bak1
        //  - With empth backup tag: shared.dat.bak
        //  - Without backup tag: user3_1.0.28561.dat
        let ext_str = 
            &self.backup_id.clone().map_or(".dat".to_owned(), 
            |x: String| format!(".dat.bak{}", x)
        );

        // Special attribute for internal file management.
        let internal_tag_str = 
          &self.internal_tag.clone().map_or("".to_owned(), 
            |x: String| format!("__{}__", x)
        );
        
        write!(f, "{internal_tag_str}user{}{ver_str}{ext_str}", &self.tag)
    }
}


#[test]
fn test_info_to_name() {

    // Default case
    let case1 = SaveNameInfo {
        tag: "1".to_owned(),
        version: None,
        backup_id: None,
        internal_tag: None,
    };

    assert_eq!(case1.to_string(), "user1.dat", "testing: basic info tag");
    
    let case2 = SaveNameInfo {
      tag: "4".to_owned(),
      version: Some("1.0.28650".to_owned()),
      ..case1.clone()
    };

    assert_eq!(case2.to_string(), "user4_1.0.28650.dat", "testing: with version tag");

    let case3 = SaveNameInfo {
      backup_id: Some("13".to_owned()),
      ..case2.clone()
    };

    assert_eq!(case3.to_string(), "user4_1.0.28650.dat.bak13", "testing: with version, backup tag");

      let case4 = SaveNameInfo {
      backup_id: Some("".to_owned()),
      ..case2.clone()
    };

    assert_eq!(case4.to_string(), "user4_1.0.28650.dat.bak", "testing: empty backup tag");

    let case5 = SaveNameInfo {
      tag: "2".to_owned(),
      backup_id: Some("15".to_owned()),
      ..case1.clone()
    };

    assert_eq!(case5.to_string(), "user2.dat.bak15", "testing: backup tag only");

    let case6 = SaveNameInfo {
      tag: "2".to_owned(),
      backup_id: Some("".to_owned()),
      ..case1.clone()
    };

  assert_eq!(case6.to_string(), "user2.dat.bak", "testing: empty backup tag only");

  let case7 = SaveNameInfo {
    internal_tag: Some("pin".to_owned()),
    ..case3.clone()
  };

  assert_eq!(case7.to_string(), "__pin__user4_1.0.28650.dat.bak13", "testing: with internal tag");

}
