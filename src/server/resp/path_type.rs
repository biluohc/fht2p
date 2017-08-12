#[derive(Debug,PartialEq)]
pub enum PathType {
    Sfs,
    File,
    Dir,
    Code,
}
impl PathType {
   pub fn is_sfs(&self) -> bool {
        match *self {
            PathType::Sfs => true,
            _ => false,
        }
    }
   pub  fn is_file(&self) -> bool {
        match *self {
            PathType::File => true,
            _ => false,
        }
    }
   pub  fn is_dir(&self) -> bool {
        match *self {
            PathType::Dir => true,
            _ => false,
        }
    }
   pub  fn is_code(&self) -> bool {
        match *self {
            PathType::Code => true,
            _ => false,
        }
    }
}