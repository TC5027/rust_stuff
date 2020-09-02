// inspired by chashmap : https://docs.rs/chashmap/2.2.2/chashmap/
use std::hash::{Hash, Hasher, BuildHasher};
use std::sync::RwLock;
use std::collections::hash_map::RandomState;
use std::sync::atomic::{AtomicUsize, Ordering};


#[derive(Debug)]
enum Container<T> {
    Empty,
    ElemRepeat(T,usize),
}

impl<T> Container<T> {
    fn is_empty(&self) -> bool {
        match self {
            Container::Empty => true,
            _ => false
        }
    }
}


#[derive(Debug)]
struct Table<T> {
    containers: Vec<RwLock<Container<T>>>,
    size: usize,
    hasher: RandomState,
}

impl<T> Table<T> {
    fn new(size: usize) -> Self {
        let containers = (0..size)
                .map(|_| RwLock::new(Container::Empty))
                .collect();
        let hasher = std::collections::hash_map::RandomState::new();
        Table { containers, size, hasher }
    }

    fn size(&self) -> usize {
        self.size
    }

    /// returns true if a Container previously empty is used, false if the value was already there
    fn add(&self, value: T, repeatitions: usize) -> bool
        where T: Hash + PartialEq
    {
        let mut hasher = self.hasher.build_hasher();
        value.hash(&mut hasher);
        let mut hash = (hasher.finish() as usize) % self.size;
        loop {
            let mut chosen = self.containers[hash].write().unwrap();
            match &*chosen {
                Container::ElemRepeat(v,r) => {
                    if v==&value {*chosen = Container::ElemRepeat(value,r+repeatitions); return false;}
                    else {hash = (hash+1)%self.size;}
                },
                Container::Empty => {*chosen = Container::ElemRepeat(value, repeatitions); return true;},
            }
        }
    }

    /// the table must never be full otherwise infinite loop possible
    fn contains(&self, value: T) -> bool 
        where T: Hash + PartialEq
    {
        let mut hasher = self.hasher.build_hasher();
        value.hash(&mut hasher);
        let mut hash = (hasher.finish() as usize) % self.size;
        loop {
            match &*self.containers[hash].read().unwrap() {
                Container::Empty => return false,
                Container::ElemRepeat(v,_) => {
                    if v==&value {return true} else {hash = (hash+1)%self.size;}
                }
            }
        }
    }

    fn double(&mut self) 
        where T: PartialEq + Hash
    {
        let mut values_already_here = vec![];
        for _ in 0..self.size {
            let rwlock = self.containers.pop().unwrap();
            let container = rwlock.into_inner().unwrap();
            if let Container::ElemRepeat(v,r) = container {
                values_already_here.push((v,r));
            }
        }
        *self = Table::new(2*self.size);
        for (v,r) in values_already_here {
            self.add(v, r);
        }
    }

    fn iteratortable(self) -> IteratorTable<T> {
        let mut containers = Vec::new();
        for rwl_c in self.containers {
            containers.push(rwl_c.into_inner().unwrap());
        }
        let current_index_containers = (0..self.size)
            .into_iter()
            .filter(|&i| !containers[i].is_empty())
            .next()
            .unwrap_or(self.size);
        let remaining_numbers_rep : usize;
        match containers[current_index_containers] {
            Container::ElemRepeat(_, repeatitions) => remaining_numbers_rep = repeatitions,
            Container::Empty => remaining_numbers_rep = 0,
        }
        IteratorTable{ containers,
                    current_index_containers,
                    remaining_numbers_rep}
    }
}

#[derive(Debug)]
pub struct CHash<T> {
    table: RwLock<Table<T>>,
    remaining: AtomicUsize,
}

impl<T> CHash<T> {
    pub fn new() -> Self {
        CHash {
                table: RwLock::new(Table::new(4)),
                remaining: AtomicUsize::new(4),}        
    }

    pub fn add(&self, value: T) 
        where T: Hash + PartialEq
    {
        if self.remaining.load(Ordering::SeqCst) < 2 {
            self.bigger();
        }
        let table = self.table.read().unwrap();
        if table.add(value,1) {
            self.remaining.fetch_sub(1, Ordering::SeqCst);
        }
    }

    fn bigger(&self) 
        where T: Hash + PartialEq
    {
        let mut table = self.table.write().unwrap();
        let size = table.size;
        table.double();
        self.remaining.fetch_add(size, Ordering::SeqCst);
    }

    pub fn contains(&self, value: T) -> bool 
        where T: Hash + PartialEq
    {
        let table  = self.table.read().unwrap();
        table.contains(value)
    }

    pub fn size(&self) -> usize {
        let table = self.table.read().unwrap();
        table.size()
    }

    pub fn iteratortable(self) -> IteratorTable<T> {
        let table = self.table.into_inner().unwrap();
        table.iteratortable()
    }
}

pub struct IteratorTable<T> {
    containers: Vec<Container<T>>,
    current_index_containers: usize,
    remaining_numbers_rep: usize,
}

impl<T: Copy> Iterator for IteratorTable<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index_containers >= self.containers.len() {
            None
        } else if self.remaining_numbers_rep>0{
            self.remaining_numbers_rep -= 1;
            match self.containers[self.current_index_containers] {
                Container::ElemRepeat(value, _) => Some(value),
                Container::Empty => None
            }
        } else {
            match (self.current_index_containers+1..self.containers.len()).into_iter().filter(|&i| !self.containers[i].is_empty()).next() {
                Some(index) => {
                    self.current_index_containers = index;
                    match self.containers[self.current_index_containers] {
                        Container::ElemRepeat(value,repeatitions) => {
                            self.remaining_numbers_rep = repeatitions-1;
                            Some(value)
                        }
                        Container::Empty => None
                    }
                }
                None => {
                    self.current_index_containers = self.containers.len();
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread::spawn;
    use std::time::Instant;
    use std::collections::HashMap;
    #[test]
    fn it_works() {
        let table = Arc::new(Table::new(4));
        let t1 = table.clone();
        let handler1 = spawn(move || {
            t1.add(0,1);
            t1.add(1,1);
        });
        let t2 = table.clone();
        let handler2 = spawn(move || {
            t2.add(1,1);
            t2.add(4,1);
        });
        handler1.join();
        handler2.join();
        assert!(table.contains(1));
        assert!(table.contains(0));
        assert!(table.contains(4));
        assert!(!table.contains(2));
    }

    #[test]
    fn iterator() {
        let table = Table::new(4);
        table.add(4,1);
        table.add(0,1);
        table.add(1,1);
        table.add(0,1);
        table.add(2,1);
        let mut output = table.iteratortable().into_iter().collect::<Vec<_>>();
        output.sort();
        assert_eq!(vec![0,0,1,2,4],output);
    }

    #[test]
    fn beaucoup() {
        let table = Arc::new(Table::new(10_000));
        let (t1,t2,t3) = (table.clone(), table.clone(), table.clone());
        let start = Instant::now();
        let h1 = spawn(move || {
            for i in 0..4000 {
                t1.add(i,1);
            }
        });
        let h2 = spawn(move || {
            for i in 4000..7000 {
                t2.add(i,1);
            }
        });
        let h3 = spawn(move || {
            for i in 7000..10000 {
                t3.add(i,1);
            }
        });
        h1.join();
        h2.join();
        h3.join();
        let duration = start.elapsed();
        // utiliser cargo test -- --nocapture pour voir les print durant test
        println!("Time elapsed for chash with 3 spawns : {:?}", duration);
        assert!(table.contains(0));
        let mut map = HashMap::new();
        let start2 = Instant::now();
        for i in 0..10_000 {
            map.insert(i,i);
        }
        let duration2 = start2.elapsed();
        println!("Time elapsed for std hashmap : {:?}",duration2);
    }

    #[test]
    fn chash_test() {
        let ch = CHash::new();
        for i in 0..6 {
            ch.add(i);
        }
        //let table = ch.table.into_inner().unwrap();
        //println!("{:?}",table.iteratortable().into_iter().collect::<Vec<_>>());
        assert!(ch.contains(0));
    }

    #[test]
    fn chash_threads_test() {
        let ch = Arc::new(CHash::new());
        let ch1 = ch.clone();
        let ch2 = ch.clone();

        let h1 = spawn(move || {
            for i in 0..10 {
                ch1.add(i);
            }
        });
        let h2 = spawn(move || {
            for i in 5..15 {
                ch2.add(i);
            }
        });

        h1.join();
        h2.join();

        println!("{:?}",ch);
        assert!(ch.contains(1));
        assert!(ch.contains(14));
    }
}
