use std::{cmp::Ordering, time::Instant};

use crate::hmac::murmur3::murmur3_32;

use super::heap::MinHeap;

const LOOKUP_TABLE_CNT: u32 = 256;

pub struct HeavyKeeper {
    k: u32,
    width: u32,
    lookups: Vec<f64>,

    // self.buckets.len() is depth
    buckets: Vec<Vec<Bucket>>,
    min_heap: MinHeap<Item>,
    last_heap_decay: Instant,
    heap_decay: usize,
    heap_decay_interval_ms: u128,
}

impl HeavyKeeper {
    pub fn new(k: u32, width: u32, depth: u32, decay: f64) -> HeavyKeeper {
        let buckets = (0..depth)
            .into_iter()
            .map(|_| {
                let mut line = Vec::with_capacity(width as usize);
                line.resize(width as usize, Bucket::default());
                line
            })
            .collect::<Vec<_>>();

        let lookups = (0..LOOKUP_TABLE_CNT)
            .map(|i| decay.powi(i as i32))
            .collect();
        HeavyKeeper {
            k,
            width,
            buckets,
            lookups,
            min_heap: MinHeap::new(),
            heap_decay: 1,
            last_heap_decay: Instant::now(),
            heap_decay_interval_ms: 1000u128,
        }
    }

    pub fn set_heap_decay_interval_ms(&mut self, heap_decay_interval_ms: u128) {
        self.heap_decay_interval_ms = heap_decay_interval_ms;
    }

    pub fn set_heap_decay(&mut self, heap_decay: usize) {
        self.heap_decay = heap_decay;
    }

    pub fn list(&self) -> Vec<Item> {
        let mut data: Vec<_> = self.min_heap.list();
        data.sort();
        data.reverse();

        data
    }

    pub fn do_heap_decay(&mut self) {
        if self.heap_decay == 1 {
            return;
        }
        if self.last_heap_decay.elapsed().as_millis() < self.heap_decay_interval_ms {
            return;
        }
        self.min_heap.replace_with(|item| {
            item.cnt = (item.cnt as usize / self.heap_decay) as u32;
        });
        self.last_heap_decay = Instant::now();
    }

    pub fn add<T: AsRef<[u8]>>(&mut self, key: T, incr: u32) -> bool {
        self.do_heap_decay();

        let key_fp = murmur3_32(key.as_ref(), 0);
        let mut max_cnt: u32 = 0;
        for (i, row) in self.buckets.iter_mut().enumerate() {
            let seed_fp = murmur3_32(key.as_ref(), i as u32) as usize;
            let bucket = seed_fp % self.width as usize;
            let fp = row[bucket].fingerprint;
            let cnt = row[bucket].cnt;

            if cnt == 0 {
                row[bucket].fingerprint = key_fp;
                row[bucket].cnt = incr;
                max_cnt = max_cnt.max(incr);
            } else if fp == key_fp {
                row[bucket].cnt += incr;
                max_cnt = max_cnt.max(row[bucket].cnt);
            } else {
                for lincr in (1..=incr).into_iter().rev() {
                    let cur_cnt = row[bucket].cnt;
                    let ldecay = if cur_cnt < LOOKUP_TABLE_CNT {
                        self.lookups[cur_cnt as usize]
                    } else {
                        self.lookups[LOOKUP_TABLE_CNT as usize - 1]
                    };

                    if rand::random::<f64>() < ldecay {
                        row[bucket].cnt -= 1;
                        if row[bucket].cnt == 0 {
                            row[bucket].fingerprint = key_fp;
                            row[bucket].cnt = lincr;
                            max_cnt = max_cnt.max(lincr);
                            break;
                        }
                    }
                }
            }
        }

        let min_heap_val = self.min_heap.peek().map(|x| x.cnt()).unwrap_or_default();

        if self.min_heap.len() == self.k as usize && max_cnt < min_heap_val {
            return false;
        }

        let finded = self.min_heap.replace_with_any(|item| {
            if item.key() != key.as_ref() {
                return false;
            }
            item.set_cnt(max_cnt);
            true
        });
        if finded {
            return true;
        }

        if self.k as usize > self.min_heap.len() {
            self.min_heap.push(Item {
                key: key.as_ref().to_vec(),
                cnt: max_cnt,
            });
        } else if max_cnt > self.min_heap.peek().map(|x| x.cnt).unwrap_or_default() {
            let _ = self.min_heap.pop();
            self.min_heap.push(Item {
                key: key.as_ref().to_vec(),
                cnt: max_cnt,
            });
        }

        true
    }
}

#[derive(Clone, Default)]
struct Bucket {
    fingerprint: u32,
    cnt: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Item {
    key: Vec<u8>,
    cnt: u32,
}

impl Item {
    fn key(&self) -> &[u8] {
        &self.key
    }

    fn cnt(&self) -> u32 {
        self.cnt
    }

    fn set_cnt(&mut self, cnt: u32) {
        self.cnt = cnt;
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.cnt < other.cnt {
            return Some(Ordering::Less);
        }
        if self.cnt != other.cnt {
            return Some(Ordering::Greater);
        }

        other.key.partial_cmp(&self.key)
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        if self < other {
            Ordering::Less
        } else if self == other {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }
}

#[test]
fn test_topk_list() {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    const N: usize = 10000;

    let fd = File::open("data/heavykeeper.data").unwrap();
    let mut br = BufReader::new(fd);
    let mut data = String::new();
    br.read_to_string(&mut data).unwrap();
    let mut iter = data.split("\n");

    let mut topk = HeavyKeeper::new(10, 10000, 5, 0.925);
    let mut rate = 0f64;

    let mut data = HashMap::new();
    for _i in 0..N {
        let key = format!("{}", iter.next().unwrap());
        let key_data = key.as_bytes().to_vec();
        let handle = data.entry(key_data.clone()).or_insert(0);
        *handle += 1;
        topk.add(key_data, 1);
    }

    for item in topk.list().into_iter() {
        rate += item.cnt as f64 - data[&item.key] as f64 / data[&item.key] as f64;
        println!(
            "item:{}, cnt:{}, expect:{}",
            String::from_utf8_lossy(item.key()),
            item.cnt(),
            data[&item.key]
        );
    }
    println!("err rate avg:{}", rate);
    for (i, node) in topk.list().into_iter().enumerate() {
        println!(
            "i:{} key:{} cnt:{}",
            i,
            String::from_utf8_lossy(node.key()),
            node.cnt()
        );
    }

    for (i, node) in topk.list().into_iter().enumerate() {
        assert_eq!(format!("{}", i), String::from_utf8_lossy(node.key()))
    }
}
