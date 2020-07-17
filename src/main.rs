use std::io::Read;
mod parser;

fn main() {
    for x in std::env::args().skip(1) {
        let mut s = String::new();
        std::fs::File::open(x)
            .expect("No FIle")
            .read_to_string(&mut s)
            .expect("No good file");
        let p = parser::section_pull(&s);
        for res in p.take(10) {
            println!("---------------");
            println!("{:?}", res);
        }
    }
    println!("Done");
}
