fn main() {
    let x = 3;
    if (x as f64) != 0.0 {
        println!("{}", "x is truthy！！！");}
    else {
        println!("{}", "x is falsy");}
    let y = 0;
    if (y as f64) != 0.0 {
        println!("{}", "y is truthy");}
    else {
        println!("{}", "y is falsy");}
}
