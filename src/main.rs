mod bin_file;
mod binary;
mod dyn_binary;
mod foreign;
mod table;
mod test;
mod test_table;

fn main() {
    println!("test_path");
    test::test_path();
    println!("test_default");
    test::test_default();
    println!("test1");
    test::test1();
    println!("test2");
    test::test2();
    println!("test_table_get");
    test_table::test_table_get();
    println!("Success!");
}
