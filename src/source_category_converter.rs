use std::{cell::RefCell, collections::HashMap, rc::Rc};

use cooplan_definitions_lib::{
    category::Category, source_attribute::SourceAttribute, source_category::SourceCategory,
};

use crate::error::{Error, ErrorKind};

pub struct SourceCategoryConverter {
    categories_map: HashMap<String, Rc<RefCell<Category>>>,
    name_id_links: HashMap<String, String>,
    root: Vec<Rc<RefCell<Category>>>,
}

impl SourceCategoryConverter {
    pub fn new() -> SourceCategoryConverter {
        SourceCategoryConverter {
            categories_map: HashMap::new(),
            name_id_links: HashMap::new(),
            root: Vec::new(),
        }
    }

    pub fn convert(
        &mut self,
        source_category: SourceCategory,
    ) -> Result<Rc<RefCell<Category>>, Error> {
        let id = match &source_category.id {
            Some(id) => id.clone(),
            None => {
                return Err(Error::new(
                    ErrorKind::MissingId,
                    format!("source category '{}' has no id", source_category.name).as_str(),
                ))
            }
        };

        self.name_id_links
            .insert(source_category.name.clone(), id.clone());

        self.convert_source_category_to_category(source_category)
    }

    fn convert_source_category_to_category(
        &mut self,
        source_category: SourceCategory,
    ) -> Result<Rc<RefCell<Category>>, Error> {
        let category_result = match &source_category.id {
            Some(_) => match &source_category.parent_name {
                Some(_) => self.create_category_from_source_with_parent(source_category),
                None => self.create_category_from_source(source_category),
            },
            None => {
                return Err(Error::new(
                    ErrorKind::MissingId,
                    format!("source category '{}' has no id", source_category.name).as_str(),
                ))
            }
        };

        match category_result {
            Ok(category) => Ok(category),
            Err(error) => Err(error),
        }
    }

    fn create_category_from_source(
        &mut self,
        source_category: SourceCategory,
    ) -> Result<Rc<RefCell<Category>>, Error> {
        let id = match source_category.id {
            Some(id) => id,
            None => {
                return Err(Error::new(
                    ErrorKind::MissingId,
                    format!("unexpected source category with no id").as_str(),
                ));
            }
        };

        let attributes = match SourceAttribute::to_attributes(source_category.attributes.as_slice())
        {
            Ok(attributes) => attributes,
            Err(error) => return Err(Error::from(error)),
        };

        let category = Category::new(
            id.clone(),
            source_category.name,
            source_category.selectable_as_last.unwrap_or(false),
            attributes,
        );

        self.categories_map.insert(id.clone(), Rc::clone(&category));
        self.root.push(Rc::clone(&category));

        Ok(category)
    }

    fn create_category_from_source_with_parent(
        &mut self,
        source_category: SourceCategory,
    ) -> Result<Rc<RefCell<Category>>, Error> {
        let id = match source_category.id {
            Some(id) => id,
            None => {
                return Err(Error::new(
                    ErrorKind::MissingId,
                    format!("unexpected source category with no id").as_str(),
                ));
            }
        };

        if source_category.parent_name.is_none() {
            return Err(Error::new(
                ErrorKind::ParentNotFound,
                format!("unexpected parentless source category").as_str(),
            ));
        }

        let parent_name = source_category.parent_name.unwrap();
        let parent_id = match self.name_id_links.get(&parent_name) {
            Some(id) => id,
            None => {
                return Err(Error::new(
                    ErrorKind::IdNotFound,
                    format!("category name '{}' is not linked with an id", parent_name).as_str(),
                ))
            }
        };

        match self.categories_map.get(parent_id) {
            Some(parent_category) => {
                let attributes =
                    match SourceAttribute::to_attributes(source_category.attributes.as_slice()) {
                        Ok(attributes) => attributes,
                        Err(error) => return Err(Error::from(error)),
                    };

                match Category::new_into_parent(
                    id,
                    Rc::downgrade(parent_category),
                    source_category.name,
                    source_category.selectable_as_last.unwrap_or(false),
                    attributes,
                ) {
                    Ok(category) => Ok(category),
                    Err(error) => Err(Error::from(error)),
                }
            }
            None => Err(Error::new(
                ErrorKind::ParentNotAvailable,
                format!("parent '{}' has not been read yet", parent_id).as_str(),
            )),
        }
    }
}
