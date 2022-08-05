// prints the type of given variable
pub fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}