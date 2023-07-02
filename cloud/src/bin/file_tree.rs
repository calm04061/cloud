use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::FileMetaType::{DIR, FILE};

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum FileMetaType {
    FILE,
    DIR,
    SYMLINK,
}

impl Display for FileMetaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FILE => f.write_str("FILE"),
            DIR => f.write_str("DIR"),
            FileMetaType::SYMLINK => f.write_str("SYMLINK"),
        }
    }
}

#[derive(Debug, Clone)]
struct FileNode {
    id: Option<i32>,
    file_type: FileMetaType,
    name: String,
    length: usize,
    version: usize,
    dirty: bool,
    children: Vec<Rc<RefCell<FileNode>>>,
}

impl FileNode {
    fn new(name: &str, file_type: FileMetaType) -> FileNode {
        FileNode {
            id: None,
            file_type,
            name: String::from(name),
            length: 0,
            version: 0,
            dirty: true,
            children: vec![],
        }
    }
}

impl Display for FileNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut temp = vec![];
        let iter = self.children.iter();
        for temp_node in iter {
            let ref_node = temp_node.borrow();
            let node = ref_node.clone();
            temp.push(node.to_string());
        }
        f.write_str("{id:").ok();
        if self.id.is_none() {
            f.write_str("None").ok();
        } else {
            f.write_str(self.id.unwrap().to_string().as_str()).ok();
        }
        f.write_str(",name:").ok();
        f.write_str(self.name.as_str()).ok();
        f.write_str(",file_type:").ok();
        f.write_str(self.file_type.to_string().as_str()).ok();
        f.write_str(",children:[").ok();
        let mut index = 0;
        for temp_node in &self.children {
            let x = temp_node.borrow();
            if index > 0 {
                f.write_str(",").ok();
            }
            f.write_str(x.clone().to_string().as_str()).ok();
            index += 1;
        }
        f.write_str("]}")
    }
}

#[derive(Debug)]
struct FileManager {
    root: Rc<RefCell<FileNode>>,

}

impl FileManager {
    fn new() -> FileManager {
        FileManager {
            root: Rc::new(RefCell::new(FileNode::new("", DIR))),
        }
    }
    fn add_node(&mut self, path: String, file_type: FileMetaType) {
        let option = path.rfind("/");
        let mut target_node = self.root.clone();
        let name;
        if option.is_none() {
            name = path.as_str();
        } else {
            let size = option.unwrap();
            let val = path.as_str();
            let parent;
            if path.starts_with("/") {
                parent = &val[1..size];
            } else {
                parent = &val[0..size];
            }
            name = &val[size + 1..];
            let split = parent.split("/");
            for temp_name in split {
                let mut target = None;
                let rc1 = target_node.clone();
                let mut ref_mut = rc1.borrow_mut();
                for temp_node in ref_mut.children.iter() {
                    let ref_node = temp_node.borrow();
                    if ref_node.name == temp_name {
                        if ref_node.file_type != DIR {
                            panic!("{} is not a dir", temp_name);
                        }
                        target = Some(temp_node);
                    }
                }
                if target.is_none() {
                    let file_node = FileNode::new(temp_name, DIR);
                    let rc = Rc::new(RefCell::new(file_node.clone()));
                    ref_mut.children.push(rc.clone());
                    target_node = rc;
                } else {
                    target_node = target.unwrap().clone();
                }
            }
        }
        let file_node = FileNode::new(name, file_type);
        let rc = Rc::new(RefCell::new(file_node.clone()));
        let mut ref_mut1 = target_node.borrow_mut();
        ref_mut1.children.push(rc.clone());
    }
}


fn main() {
    let mut manager = FileManager::new();
    manager.add_node("/test/123/123".to_string(), DIR);
    println!("{:?}", manager.root.borrow().clone().to_string());
    manager.add_node("/test/123/123/test".to_string(), DIR);
    println!("{:?}", manager.root.borrow().clone().to_string());
}
