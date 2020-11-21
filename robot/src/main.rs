// There is a robot starting at position (0, 0), the origin, on a 2D plane. Given a sequence of its moves
// judge if this robot ends up at (0, 0) after it completes its moves.

use rayon::prelude::*;

struct Position {
    x : i32,
    y : i32,
}

impl Position {
    pub fn new() -> Self {
        Position {
            x : 0,
            y : 0,
        }
    }
}

use std::iter::Sum;
impl Sum for Position {
    fn sum<I> (iter: I) -> Self 
        where I: Iterator<Item = Self>
    {
        iter.fold(Self::new(), |a, b| Self {
            x: a.x + b.x,
            y: a.y + b.y
        })
    }
}

use std::convert::From;
impl From<char> for Position {
    fn from(c: char) -> Self {
        match c {
            'U' => Position {x:0, y:-1},
            'D' => Position {x:0, y:1},
            'R' => Position {x:1, y:0},
            'L' => Position {x:-1, y:0},
            _ => Position::new()
        }
    }
}


pub fn judge_circle(moves: String) -> bool {
    let end : Position = moves
        .par_chars()
        .map(|c| Position::from(c))
        .sum();
    end.x==0 && end.y==0
}

use std::collections::HashMap;
use std::sync::Mutex;
pub fn judge_circle_lock(moves: String) -> bool {
    let hm = Mutex::new(HashMap::new());
    moves.par_chars().for_each(|c| {
        *hm.lock().unwrap().entry(c).or_insert(0) += 1;
    });
    let hm = hm.into_inner().unwrap();
    hm.get(&'U').unwrap_or(&0)==hm.get(&'D').unwrap_or(&0) && hm.get(&'L').unwrap_or(&0)==hm.get(&'R').unwrap_or(&0)
}

use std::sync::atomic::{AtomicBool,AtomicI32,AtomicUsize,Ordering};
pub fn tool(mut moves : String, keep_going : &AtomicBool, x : &AtomicI32, y : &AtomicI32, remaining_size : &AtomicUsize) {
    if keep_going.load(Ordering::SeqCst) {
        let len = moves.len();
        if len<=20_000 {
            let (dx, dy) = moves.chars().map(|c| {
                match c {
                    'U' => (0,-1),
                    'D' => (0,1),
                    'L' => (-1,0),
                    'R' => (1,0),
                    _ => (0,0),
                }
            }).fold((0,0), |(a,b),(dx,dy)| (a+dx,b+dy));
            if (x.load(Ordering::SeqCst)+dx).abs() as usize > (remaining_size.load(Ordering::SeqCst)-len) || (y.load(Ordering::SeqCst)+dy).abs() as usize > (remaining_size.load(Ordering::SeqCst)-len)
            {
                keep_going.store(false, Ordering::SeqCst);
                x.fetch_add(dx, Ordering::SeqCst);
                y.fetch_add(dy, Ordering::SeqCst);
            } else {
                remaining_size.fetch_sub(len, Ordering::SeqCst);
                x.fetch_add(dx, Ordering::SeqCst);
                y.fetch_add(dy, Ordering::SeqCst);
            }
        } else {
            let snd_half = moves.split_off(len/2);
            rayon::join(
                || {tool(moves, keep_going, x, y, remaining_size);},
                || {tool(snd_half, keep_going, x, y, remaining_size);}
            );
        }
    }
}
pub fn early_stop(moves: String) -> bool {
    let keep_going = AtomicBool::new(true);
    let x = AtomicI32::new(0);
    let y = AtomicI32::new(0);
    let remaining_size = AtomicUsize::new(moves.len());
    tool(moves, &keep_going, &x, &y, &remaining_size);
    x.load(Ordering::SeqCst)==0 && y.load(Ordering::SeqCst)==0
}

pub fn leetcode_accepted(moves: String) -> bool {
    let check = moves.chars().map(|c| {
        match c {
            'U' => (0,-1),
            'D' => (0,1),
            'L' => (-1,0),
            'R' => (1,0),
            _ => (0,0),
        }
    }).fold((0,0), |(a,b),(dx,dy)| (a+dx,b+dy));
    check.0==0 && check.1==0
}


use rand::{thread_rng, Rng};
pub fn generate(length:usize) -> String {
    let mut rng = thread_rng();
    (0..length).map(|_| match rng.gen_range(0,4) {
        0 => 'U',
        1 => 'D',
        2 => 'L',
        _ => 'R',
    }).collect()
}


fn main() {
    let moves = generate(500_000);
    let start = std::time::Instant::now();
    //fast_tracer::svg("visu.svg", || judge_circle(moves));
    judge_circle(moves);
    println!("it took {:?}", start.elapsed());
}
 