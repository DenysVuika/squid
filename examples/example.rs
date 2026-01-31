use std::fs::File;
use std::io::Read;

// Example Rust code with intentional issues for code review testing

pub struct UserData {
    pub name: String,
    pub age: i32,
    pub email: String,
}

// Issue: Using unwrap() which can panic
pub fn read_config(path: &str) -> String {
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents
}

// Issue: Using clone unnecessarily
pub fn process_user(user: UserData) -> UserData {
    let name = user.name.clone();
    let email = user.email.clone();

    UserData {
        name: name,
        email: email,
        age: user.age,
    }
}

// Issue: Not idiomatic, could use iterator
pub fn sum_numbers(numbers: Vec<i32>) -> i32 {
    let mut sum = 0;
    for i in 0..numbers.len() {
        sum += numbers[i];
    }
    sum
}

// Issue: Using String when &str would be better
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

// Issue: No error handling
pub fn divide(a: i32, b: i32) -> i32 {
    a / b
}

// Issue: Overly nested code
pub fn check_value(value: Option<i32>) -> String {
    if value.is_some() {
        let v = value.unwrap();
        if v > 0 {
            if v < 100 {
                return "Valid".to_string();
            } else {
                return "Too large".to_string();
            }
        } else {
            return "Negative".to_string();
        }
    } else {
        return "None".to_string();
    }
}

// Issue: Magic numbers
pub fn calculate_price(quantity: i32) -> f64 {
    if quantity > 10 {
        quantity as f64 * 9.99 * 0.9
    } else {
        quantity as f64 * 9.99
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum() {
        assert_eq!(sum_numbers(vec![1, 2, 3]), 6);
    }
}
