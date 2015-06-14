#![crate_type = "plugin"]
pub fn foo(x: u32) -> String { format!("Hello world! #{}", x) }
