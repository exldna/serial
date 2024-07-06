#[test]
fn test_basic() {
    let mut evil = serial::LinkedList::<i32>::new();
    assert_eq!(evil.len(), 0);
    evil.push_front(1);
    evil.push_front(2);
    evil.push_front(3);
    assert_eq!(evil.len(), 3);
    assert_eq!(evil.pop_back(), Some(1));
    assert_eq!(evil.pop_back(), Some(2));
    assert_eq!(evil.pop_back(), Some(3));
    assert_eq!(evil.pop_back(), None);
    assert_eq!(evil.len(), 0);
}

#[test]
#[allow(dead_code)]
fn assert_covariance() {
    fn a<'a>(x: serial::LinkedList<&'static str>) -> serial::LinkedList<&'a str> {
        x
    }
}

#[test]
fn test_dropck() {
    let mock = utils::DropMock::new();
    {
        let mut evil = serial::LinkedList::<utils::DropMock>::new();
        for _ in 0..10 {
            evil.push_back(mock.clone());
        }
    }
    assert_eq!(mock.drop_cnt(), 10);
}

#[test]
fn test_too_many_lists_basic_front() {
    let mut list = serial::LinkedList::new();

    // Try to break an empty list
    assert_eq!(list.len(), 0);
    assert_eq!(list.pop_front(), None);
    assert_eq!(list.len(), 0);

    // Try to break a one item list
    list.push_front(10);
    assert_eq!(list.len(), 1);
    assert_eq!(list.pop_front(), Some(10));
    assert_eq!(list.len(), 0);
    assert_eq!(list.pop_front(), None);
    assert_eq!(list.len(), 0);

    // Mess around
    list.push_front(10);
    assert_eq!(list.len(), 1);
    list.push_front(20);
    assert_eq!(list.len(), 2);
    list.push_front(30);
    assert_eq!(list.len(), 3);
    assert_eq!(list.pop_front(), Some(30));
    assert_eq!(list.len(), 2);
    list.push_front(40);
    assert_eq!(list.len(), 3);
    assert_eq!(list.pop_front(), Some(40));
    assert_eq!(list.len(), 2);
    assert_eq!(list.pop_front(), Some(20));
    assert_eq!(list.len(), 1);
    assert_eq!(list.pop_front(), Some(10));
    assert_eq!(list.len(), 0);
    assert_eq!(list.pop_front(), None);
    assert_eq!(list.len(), 0);
    assert_eq!(list.pop_front(), None);
    assert_eq!(list.len(), 0);
}

#[test]
fn test_too_many_lists_basic() {
    let mut m = serial::LinkedList::new();
    assert_eq!(m.pop_front(), None);
    assert_eq!(m.pop_back(), None);
    assert_eq!(m.pop_front(), None);
    m.push_front(1);
    assert_eq!(m.pop_front(), Some(1));
    m.push_back(2);
    m.push_back(3);
    assert_eq!(m.len(), 2);
    assert_eq!(m.pop_front(), Some(2));
    assert_eq!(m.pop_front(), Some(3));
    assert_eq!(m.len(), 0);
    assert_eq!(m.pop_front(), None);
    m.push_back(1);
    m.push_back(3);
    m.push_back(5);
    m.push_back(7);
    assert_eq!(m.pop_front(), Some(1));

    let mut n = serial::LinkedList::new();
    n.push_front(2);
    n.push_front(3);
    {
        assert_eq!(n.front().unwrap(), &3);
        let x = n.front_mut().unwrap();
        assert_eq!(*x, 3);
        *x = 0;
    }
    {
        assert_eq!(n.back().unwrap(), &2);
        let y = n.back_mut().unwrap();
        assert_eq!(*y, 2);
        *y = 1;
    }
    assert_eq!(n.pop_front(), Some(0));
    assert_eq!(n.pop_front(), Some(1));
}
