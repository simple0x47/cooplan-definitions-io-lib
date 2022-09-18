use std::cell::RefCell;
use std::io::Error;
use std::rc::Rc;

use cooplan_definitions_lib::category::Category;
use cooplan_definitions_lib::source_category::SourceCategory;

/// Abstract wrapping for interacting with the storing of categories.
pub trait CategoryIO {
    fn read(&mut self) -> Result<SourceCategory, Error>;
    fn write(&self, category: &Rc<RefCell<Category>>) -> Result<(), Error>;
    fn parent_name(&self) -> Result<Option<String>, Error>;
}
