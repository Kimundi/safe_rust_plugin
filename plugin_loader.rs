#![feature(std_misc, core)]

use std::dynamic_lib::DynamicLibrary;
use std::path::Path;
use std::slice;
use std::mem;
use std::any::TypeId;
use std::marker::Reflect;

type PluginTypeInfo = &'static [(TypeId, &'static str, *const u8)];

struct CheckedDynLib {
    handle: Option<DynamicLibrary>,
    type_info: PluginTypeInfo,
}

impl CheckedDynLib {
    fn open(path: &Path) -> Result<Self, String> {
        let handle = try!(DynamicLibrary::open(Some(path)));

        // Load type info symbol and cache it in the struct

        let type_info: PluginTypeInfo = unsafe {
            let symbol = try!(handle.symbol::<usize>("rust_plugin_typeinfo"));

            // symbol points at a size field which is followed by the array.
            let array_len = *symbol;
            let array_ptr = symbol.offset(1);

            slice::from_raw_parts(array_ptr as *const _, array_len)
        };

        Ok(CheckedDynLib {
            handle: Some(handle),
            type_info: type_info,
        })
    }

    fn lookup_fn<T: Copy + Reflect + 'static>(&self, module_path: &str) -> Result<T, String> {
        let type_id = TypeId::of::<T>();

        // search the type_id in the type_info table
        self.type_info.iter()
            .filter(|t| t.0 == type_id && t.1 == module_path)
            .map(|t| unsafe {
                *(&t.2 as *const _ as *const T)
            })
            .next()
            .ok_or("item does not exist".into())
    }
}

impl Drop for CheckedDynLib {
    fn drop(&mut self) {
        // Leak the handle so that function pointers never become invalid
        mem::forget(self.handle.take());
    }
}

#[test]
fn it_works1() {
    let lib = CheckedDynLib::open(Path::new("./libexample_plugin.so")).unwrap();
    let f = lib.lookup_fn::<fn(u32) -> String>("foo").unwrap();
    println!("{}", f(42));
}

#[test]
#[should_panic]
fn it_works2() {
    let lib = CheckedDynLib::open(Path::new("./libexample_plugin.so")).unwrap();

    let g = lib.lookup_fn::<fn(u32) -> String>("bar").unwrap();
    println!("{}", g(42));
}

#[test]
#[should_panic]
fn it_works3() {
    let lib = CheckedDynLib::open(Path::new("./libexample_plugin.so")).unwrap();

    let h = lib.lookup_fn::<fn(i32) -> u8>("foo").unwrap();
    println!("{}", h(42));
}
