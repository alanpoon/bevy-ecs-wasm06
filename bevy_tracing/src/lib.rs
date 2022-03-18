#[macro_export]
macro_rules! info_span {
    () => (println!("\n"));
    ($($arg:tt)*) => ({
      println!($($arg)*);
    })
}
#[macro_export]
macro_rules! info {
    () => (println!("\n"));
    ($($arg:tt)*) => ({
      println!($($arg)*);
    })
}
#[macro_export]
macro_rules! warn {
    () => (println!("\n"));
    ($($arg:tt)*) => ({
      println!($($arg)*);
    })
}
#[macro_export]
macro_rules! error {
    () => (println!("\n"));
    ($($arg:tt)*) => ({
      println!($($arg)*);
    })
}
#[macro_export]
macro_rules! trace {
    () => (println!("\n"));
    ($($arg:tt)*) => ({
      println!($($arg)*);
    })
}