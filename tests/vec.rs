#[test]
fn test_basic() {
    let mut vec = serial::Vec::<i32>::new();
    assert_eq!(vec.len(), 0);
    vec.push(1);
    vec.push(2);
    vec.push(3);
    assert_eq!(vec.len(), 3);
    assert_eq!(vec[0], 1);
    assert_eq!(vec[1], 2);
    assert_eq!(vec[2], 3);
    vec.remove(0);
    vec.remove(1);
    vec.remove(0);
    assert_eq!(vec.len(), 0);
}

#[test]
#[allow(dead_code)]
fn assert_covariance() {
    fn a<'a>(x: serial::Vec<&'static str>) -> serial::Vec<&'a str> {
        x
    }
}

#[test]
fn test_dropck() {
    let mock = utils::DropMock::new();
    {
        let mut vec = serial::Vec::<Box<utils::DropMock>>::with_capacity(10);
        for _ in 0..10 {
            vec.push(Box::new(mock.clone()));
        }
        for _ in 0..5 {
            vec.remove(0);
        }
    }
    assert_eq!(mock.drop_cnt(), 10);
}

#[test]
fn test_append() {
    let mut vec = serial::Vec::<i32>::with_capacity(5);
    for i in 0..5 {
        vec.push(i);
    }
    let mut other = serial::Vec::<i32>::with_capacity(5);
    for i in 5..10 {
        other.push(i);
    }
    let old_len = vec.len();
    vec.append(&mut other);
    assert_eq!(vec.len(), old_len + other.len());
    for (i, v) in vec.iter().enumerate() {
        assert_eq!(i as i32, *v);
    }
}

#[test]
fn test_insert() {
    let mut vec = serial::Vec::<i32>::with_capacity(5);
    vec.push(0);
    vec.push(1);
    vec.push(3);
    vec.insert(2, 2);
    vec.insert(4, 4);
    for (i, v) in vec.iter().enumerate() {
        assert_eq!(i as i32, *v);
    }
}
