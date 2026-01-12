use crate::{prelude::*, test_bin_file};

#[test]
pub fn test_cache() {
    let mut cache = Cache::new();
    assert_eq!(vec![(false, 0..=usize::MAX)], cache.chunks());
    assert!(cache.get(0).is_none());
    assert!(cache.gets(0, None).is_none());
    assert!(cache.is_empty());

    cache.inserts(0, [4, 9, 1].into_iter());
    assert_eq!(vec![(true, 0..=2), (false, 3..=usize::MAX)], cache.chunks());
    assert_eq!(Some(9), cache.get(1));
    assert_eq!(Some(vec![4, 9, 1]), cache.gets(0, None));
    assert!(!cache.is_empty());

    cache.inserts(5, [5, 0, 3].into_iter());
    assert_eq!(
        vec![
            (true, 0..=2),
            (false, 3..=4),
            (true, 5..=7),
            (false, 8..=usize::MAX)
        ],
        cache.chunks()
    );
    assert_eq!(Some(5), cache.get(5));
    assert_eq!(None, cache.get(4));
    assert_eq!(Some(vec![4, 9, 1]), cache.gets(0, Some(3)));
    assert_eq!(Some(vec![4, 9, 1, 5, 0, 3]), cache.gets(0, None));
    assert_eq!(Some(vec![5, 0, 3]), cache.gets(5, Some(3)));

    cache.inserts(3, [6, 9].into_iter());
    assert_eq!(vec![(true, 0..=7), (false, 8..=usize::MAX)], cache.chunks());
    assert_eq!(Some(5), cache.get(5));
    assert_eq!(Some(9), cache.get(4));
    assert_eq!(Some(vec![4, 9, 1, 6, 9, 5, 0, 3]), cache.gets(0, None));

    unsafe {
        cache.move_cache(3, 3);
    }
    assert_eq!(
        vec![
            (true, 0..=2),
            (false, 3..=5),
            (true, 6..=10),
            (false, 11..=usize::MAX)
        ],
        cache.chunks()
    );
    assert_eq!(None, cache.get(5));
    assert_eq!(None, cache.get(4));
    assert_eq!(Some(5), cache.get(8));
    assert_eq!(Some(9), cache.get(7));
    assert_eq!(Some(vec![4, 9, 1, 6, 9, 5, 0, 3]), cache.gets(0, None));

    unsafe {
        cache.move_cache(3, -3);
    }
    assert_eq!(vec![(true, 0..=7), (false, 8..=usize::MAX)], cache.chunks());
    assert_eq!(Some(5), cache.get(5));
    assert_eq!(Some(9), cache.get(4));
    assert_eq!(Some(vec![4, 9, 1, 6, 9, 5, 0, 3]), cache.gets(0, None));
}

#[test]
pub fn test1() {
    test_bin_file::base_test1(BDPath::new_main_str("test/test_cache1"), |path| {
        CachedBinFile::new(path.clone()).expect("failed to new")
    });
}

#[test]
pub fn test2() {
    test_bin_file::base_test2(BDPath::new_main_str("test/test_cache2"), |path| {
        CachedBinFile::new(path.clone()).expect("failed to new")
    });
}
