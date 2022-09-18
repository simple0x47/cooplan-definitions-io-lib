use std::cell::RefCell;
use std::fs::DirEntry;
use std::io::{Error, ErrorKind};
use std::rc::Rc;

use cooplan_definitions_lib::attribute::Attribute;
use cooplan_definitions_lib::category::Category;
use cooplan_definitions_lib::source_category::SourceCategory;

use crate::category_io::CategoryIO;

/// Abstract wrapping of a file containing the definition of a category.
pub struct CategoryFileIO {
    root: String,
    path: String,
}

impl CategoryFileIO {
    pub fn new(root: String, path: String) -> CategoryFileIO {
        CategoryFileIO { root, path }
    }

    pub fn root(&self) -> &str {
        self.root.as_str()
    }

    pub fn path(&self) -> &str {
        self.path.as_str()
    }
}

impl CategoryIO for CategoryFileIO {
    fn read(&mut self) -> Result<SourceCategory, Error> {
        match std::fs::read_to_string(self.path.clone()) {
            Ok(json_category) => {
                let category_result: Result<SourceCategory, serde_json::Error> =
                    serde_json::from_str(json_category.as_str());
                match category_result {
                    Ok(mut category) => {
                        match self.parent_name() {
                            Ok(parent_name_nullable) => category.parent_name = parent_name_nullable,
                            Err(error) => return Err(error),
                        }

                        return Ok(category);
                    }
                    Err(error) => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("[{}] {}", self.path(), error),
                        ))
                    }
                }
            }
            Err(error) => {
                return Err(Error::new(
                    error.kind(),
                    format!("[{}] {}", self.path(), error),
                ));
            }
        }
    }

    fn write(&self, category: &Rc<RefCell<Category>>) -> Result<(), Error> {
        match category.try_borrow() {
            Ok(borrow) => {
                let source_category = match &borrow.parent {
                    Some(parent) => SourceCategory {
                        id: Some(borrow.id.clone()),
                        parent: Some(parent.upgrade().unwrap().try_borrow().unwrap().id.clone()),
                        parent_name: Some(
                            parent.upgrade().unwrap().try_borrow().unwrap().name.clone(),
                        ),
                        name: borrow.name.clone(),
                        selectable_as_last: Some(borrow.selectable_as_last),
                        attributes: Attribute::to_source_attributes(borrow.attributes.as_slice()),
                    },
                    None => SourceCategory {
                        id: Some(borrow.id.clone()),
                        parent: None,
                        parent_name: None,
                        name: borrow.name.clone(),
                        selectable_as_last: Some(borrow.selectable_as_last),
                        attributes: Attribute::to_source_attributes(borrow.attributes.as_slice()),
                    },
                };

                match serde_json::to_string_pretty(&source_category) {
                    Ok(json_category) => std::fs::write(self.path.clone(), json_category),
                    Err(error) => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("[{}] {}", self.path(), error),
                        ));
                    }
                }
            }
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::Interrupted,
                    format!("[{}] failed to borrow category: {}", self.path(), error),
                ))
            }
        }
    }

    /// Retrieves the category's parent name based on the parent directory.
    ///
    /// # Returns
    ///
    /// * `Ok`: parent's name string or none if category has no parent.
    /// * `Err`: contains the error that occurred.
    fn parent_name(&self) -> Result<Option<String>, Error> {
        let path = self.path.clone();

        let res = path.trim_start_matches(&self.root);
        let split: Vec<&str> = res.split("/").collect();

        let mut prev_file: usize = 0;
        for s in split.as_slice() {
            if s.contains(".json") {
                break;
            }

            prev_file += 1;
        }

        if prev_file > 0 {
            return Ok(Some(split.get(prev_file - 1).unwrap().to_string()));
        }

        Ok(None)
    }
}

fn build_for_path(root: &str, dir: &str) -> Result<Vec<Box<dyn CategoryIO>>, Error> {
    const CATEGORY_FILE_EXTENSION: &str = ".json";

    let mut categories_files_io: Vec<Box<dyn CategoryIO>> = Vec::new();
    match std::fs::read_dir(dir) {
        Ok(read) => {
            let mut directories: Vec<DirEntry> = Vec::new();

            for entry_result in read {
                let entry = entry_result.unwrap();

                match entry.file_type() {
                    Ok(file_type) => match entry.path().to_str() {
                        Some(path) => {
                            if file_type.is_dir() {
                                directories.push(entry);
                            } else if path.ends_with(CATEGORY_FILE_EXTENSION) {
                                categories_files_io.push(Box::new(CategoryFileIO::new(
                                    root.to_string(),
                                    path.to_string(),
                                )));
                            }
                        }
                        None => {
                            return Err(Error::new(
                                ErrorKind::Unsupported,
                                "could not convert path to string.",
                            ));
                        }
                    },
                    Err(error) => return Err(error),
                }
            }

            for directory in directories {
                match directory.path().to_str() {
                    Some(path) => {
                        match build_for_path(root, path) {
                            Ok(mut child_categories_files_io) => {
                                let current_size = categories_files_io.len();
                                let child_size = child_categories_files_io.len();

                                // Detect overflows.
                                if current_size.checked_add(child_size).is_none() {
                                    return Err(Error::new(
                                        ErrorKind::Other,
                                        "cannot append vectors due to overflow",
                                    ));
                                }

                                categories_files_io.append(&mut child_categories_files_io);
                            }
                            Err(error) => return Err(error),
                        }
                    }
                    None => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "failed to retrieve path of directory".to_string(),
                        ));
                    }
                }
            }

            Ok(categories_files_io)
        }
        Err(error) => Err(error),
    }
}

/// Creates instances of `CategoryFileIO` for each file that has been found to be a category definition.
///
/// # Returns
///
/// `Ok`: vector containing instances of `CategoryFileIO` for each category definition file that has been found.
/// `Err`: error detailing why the function has failed.
pub fn build_for_all_categories(
    categories_directory: String,
) -> Result<Vec<Box<dyn CategoryIO>>, Error> {
    build_for_path(categories_directory.as_str(), categories_directory.as_str())
}
