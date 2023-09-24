pub fn median(list: Vec<f64>) -> f64 {
    let sum: f64 = list.iter().sum();

    let average: f64 = if list.len() > 0 {
        sum as f64 / list.len() as f64
    } else {
        0.0
    };

    average
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_median_empty_list() {
        let list: Vec<f64> = vec![];
        let result = median(list);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_median_single_element() {
        let list: Vec<f64> = vec![42.0];
        let result = median(list);
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_median_even_length_list() {
        let list: Vec<f64> = vec![10.0, 20.0, 30.0, 40.0];
        let result = median(list);
        assert_eq!(result, 25.0);
    }

    #[test]
    fn test_median_odd_length_list() {
        let list: Vec<f64> = vec![15.0, 25.0, 35.0];
        let result = median(list);
        assert_eq!(result, 25.0);
    }
}
