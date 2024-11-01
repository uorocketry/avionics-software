use crate::voltage_to_celsius;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voltage_to_celsius_test1() {
        //println!("Test 1: {}", voltage_to_celsius(20.644));
        let result:f64 = voltage_to_celsius(20.644);
        assert!(499.97 <= result && 500.0 >= result);
    }

    #[test]
    fn voltage_to_celsius_test2() {
        // println!("Test 2: {}", voltage_to_celsius(6.138));
        let result:f64 = voltage_to_celsius(6.138);
        assert!(150.01 <= result && 150.03 >= result);
    }

    #[test]
    fn voltage_to_celsius_test3() {
        // println!("Test 3: {}", voltage_to_celsius(0.039));
        let result:f64 = voltage_to_celsius(0.039);
        assert!(0.97 <= result && 0.98 >= result);
    }

    #[test]
    fn voltage_to_celsius_test4() {
        // println!("Test 4: {}", voltage_to_celsius(-0.778));
        let result:f64 = voltage_to_celsius(-0.778);
        assert!(-20.03 <= result && -20.01 >= result);
    }

    #[test]
    fn voltage_to_celsius_test5() {
        // println!("Test 5: {}", voltage_to_celsius(10.0));
        let result:f64 = voltage_to_celsius(10.0);
        assert!(246.1 <= result && 246.3 >= result);
    }
}