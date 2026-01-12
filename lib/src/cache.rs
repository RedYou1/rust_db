use std::{cmp::Ordering, ops::RangeInclusive};

use crate::add_size;

struct CacheNode<Row> {
    from: usize,
    to: usize,
    data: Vec<Row>,
}

impl<Row> CacheNode<Row> {
    pub const fn contains_index(&self, index: usize) -> Ordering {
        if index < self.from {
            Ordering::Greater
        } else if self.to < index {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

#[derive(Default)]
pub struct Cache<Row: Clone> {
    len: usize,
    cache: Vec<CacheNode<Row>>,
}

impl<Row: Clone> Cache<Row> {
    pub const fn new() -> Self {
        Self {
            len: 0,
            cache: Vec::new(),
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    fn frind_first_cache(&self, index: usize) -> Result<usize, usize> {
        if self.cache.is_empty() {
            return Err(0);
        }
        let mut from = 0;
        let mut to = self.cache.len() - 1;
        while from <= to {
            let idx = (to - from) / 2 + from;
            match self.cache[idx].contains_index(index) {
                Ordering::Equal => return Ok(idx),
                Ordering::Greater if idx == from => return Err(idx),
                Ordering::Greater => to = idx - 1,

                Ordering::Less if idx == to => return Err(idx + 1),
                Ordering::Less => from = idx + 1,
            }
        }
        panic!("cache went outside of range")
    }

    /// bool is for if is in cache
    pub fn chunks(&self) -> Vec<(bool, RangeInclusive<usize>)> {
        let mut data = Vec::with_capacity(self.cache.len() * 2 + 1);
        let mut prev: Option<(bool, RangeInclusive<usize>)> = None;
        for cache in &self.cache {
            if let Some(prev) = prev {
                data.push((false, (prev.1.end() + 1)..=(cache.from - 1)));
            } else if cache.from > 0 {
                data.push((false, 0..=(cache.from - 1)));
            }
            data.push((true, cache.from..=cache.to));
            prev = Some((true, cache.from..=cache.to));
        }
        if data.is_empty() {
            data.push((false, 0..=usize::MAX));
        } else if let Some(prev) = prev {
            data.push((false, (prev.1.end() + 1)..=usize::MAX));
        }
        data
    }

    pub fn get(&self, index: usize) -> Option<Row> {
        self.frind_first_cache(index).ok().map(|i| {
            let cache = &self.cache[i];
            cache.data[index - cache.from].clone()
        })
    }

    pub fn gets(&self, index: usize, len: Option<usize>) -> Option<Vec<Row>> {
        let i = self.frind_first_cache(index).ok()?;

        if let Some(len) = len {
            let max = index + len;
            Some(
                self.cache[i..]
                    .iter()
                    .skip_while(|c| c.to < index)
                    .take_while(|c| c.from < max)
                    .flat_map(|c| {
                        let start = index.saturating_sub(c.from);
                        &c.data[start..(max - c.from)]
                    })
                    .cloned()
                    .collect(),
            )
        } else {
            Some(
                self.cache[i..]
                    .iter()
                    .flat_map(|c| {
                        if c.from <= index {
                            &c.data[(index - c.from)..]
                        } else {
                            &c.data
                        }
                    })
                    .cloned()
                    .collect(),
            )
        }
    }

    /// # Safety
    /// Their isn't any validation so you can corrupt the data structure.
    pub unsafe fn move_cache(&mut self, index: usize, amount: isize) {
        let mut i = self.frind_first_cache(index).unwrap_or(0);
        while i < self.cache.len() {
            let cache = &mut self.cache[i];
            if cache.to >= index {
                cache.to = add_size(cache.to, amount);
                if cache.from >= index {
                    cache.from = add_size(cache.from, amount);
                    if i > 0 && cache.from == self.cache[i - 1].to + 1 {
                        self.cache[i - 1].to = self.cache[i].to;
                        let mut data = self.cache.remove(i).data;
                        self.cache[i - 1].data.append(&mut data);
                        continue;
                    }
                } else {
                    let (left, right) = cache.data.split_at(index - cache.from);
                    let (left, right) = (Vec::from(left), Vec::from(right));
                    cache.data = left;
                    let to = cache.to;
                    cache.to = index - 1;
                    self.cache.insert(
                        i + 1,
                        CacheNode {
                            from: add_size(index, amount),
                            to,
                            data: right,
                        },
                    );
                    break;
                }
            }
            i += 1;
        }
    }

    pub fn insert(&mut self, index: usize, data: Row) {
        match self.frind_first_cache(index) {
            Ok(_) => panic!("insert into cache already exists"),
            Err(i) => {
                if i > 0 && self.cache[i - 1].to + 1 == index {
                    self.cache[i - 1].to += 1;
                    self.cache[i - 1].data.push(data.clone());
                } else if self.cache.len() > i && self.cache[i].from == index + 1 {
                    self.cache[i].from -= 1;
                    self.cache[i].data.insert(0, data.clone());
                } else {
                    self.cache.insert(
                        i,
                        CacheNode {
                            from: index,
                            to: index,
                            data: vec![data.clone()],
                        },
                    );
                }
                self.len += 1;
            }
        }
    }

    pub fn inserts(&mut self, index: usize, datas: impl Iterator<Item = Row>) {
        match self.frind_first_cache(index) {
            Ok(_) => panic!("insert into cache already exists"),
            Err(i) => {
                if i > 0 && self.cache[i - 1].to + 1 == index {
                    let cache = &mut self.cache[i - 1];
                    for data in datas {
                        self.len += 1;
                        cache.to += 1;
                        cache.data.push(data.clone());
                    }
                    if i < self.cache.len() && self.cache[i - 1].to + 1 == self.cache[i].from {
                        self.cache[i - 1].to = self.cache[i].to;
                        let mut data = self.cache.remove(i).data;
                        self.cache[i - 1].data.append(&mut data);
                    }
                } else {
                    let datas: Vec<Row> = datas.collect();
                    self.len += datas.len();
                    if self.cache.len() > i && self.cache[i].from == index + datas.len() {
                        self.cache[i].from -= datas.len();
                        self.cache[i].data.splice(0..0, datas);
                    } else {
                        self.cache.insert(
                            i,
                            CacheNode {
                                from: index,
                                to: index + datas.len() - 1,
                                data: datas,
                            },
                        );
                    }
                }
            }
        }
    }

    pub fn remove(&mut self, index: usize, len: Option<usize>) {
        match (len, self.frind_first_cache(index)) {
            (None, Ok(mut i)) | (None, Err(mut i)) => {
                if let Some(cache) = self.cache.get_mut(i)
                    && cache.from < index
                {
                    self.len -= cache.to - cache.from + 1;
                    i += 1;
                    cache.to = index - 1;
                    cache.data.splice((index - cache.from).., []);
                }
                for _ in i..self.cache.len() {
                    let cache = self.cache.pop().expect("len already checked");
                    self.len -= cache.to - cache.from + 1;
                }
            }
            (_, Ok(mut i)) | (_, Err(mut i)) => {
                let len = len.unwrap_or(usize::MAX);
                if len == 0 {
                    panic!("len is 0")
                }
                loop {
                    if let Some(cache) = self.cache.get_mut(i) {
                        if cache.from + 1 > index + len {
                            break;
                        }
                        if cache.from >= index {
                            if cache.to < index + len {
                                self.len -= cache.to - cache.from + 1;
                                self.cache.remove(i);
                                continue;
                            } else {
                                self.len -= index + len - cache.from;
                                cache.data.splice(..(index + len - cache.from), []);
                                cache.from = index + len;
                                break;
                            }
                        }

                        if cache.to < index + len {
                            self.len -= cache.to - index + 1;
                            i += 1;
                            cache.to = index - 1;
                            cache.data.splice((index - cache.from).., []);
                            continue;
                        } else {
                            self.len -= len;
                            let (left, right) = cache.data.split_at(index - cache.from);
                            let (left, right) = (Vec::from(left), Vec::from(&right[len..]));
                            let (from, to) = (cache.from, cache.to);
                            self.cache.remove(i);
                            self.cache.insert(
                                i,
                                CacheNode {
                                    from,
                                    to: index - 1,
                                    data: left,
                                },
                            );
                            self.cache.insert(
                                i + 1,
                                CacheNode {
                                    from: index + len,
                                    to,
                                    data: right,
                                },
                            );
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.cache.clear();
    }
}
