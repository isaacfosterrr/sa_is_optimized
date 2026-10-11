use sa_linear::*;
use std::io;


use std::time::Instant;

fn main() {
    let data = std::fs::read("random.bin").unwrap();
    let t = Instant::now();
    let sa = build_sa_is(&data, 0);
    println!("n = {}, time = {:?}, sa[0] = {}", data.len(), t.elapsed(), sa[0]);
}
// fn  main() {
    // let mut n_s = String::new();
    // io::stdin().read_line(&mut n_s).unwrap();
    // let mut s = String::new();
    // io::stdin().read_line(&mut s).unwrap();

    // let n = n_s.trim().parse::<usize>().unwrap();
    // let bytes: Vec<u8> = s.into_bytes();
    // let sa = build_sa_is(&bytes);

    // // Remove sentinel (its index is always 0 or last in SA-IS)
    // let suffix_array: Vec<usize> = sa.into_iter().filter(|&x| x != bytes.len() - 1).collect();
    // let mut ranks = vec![0usize; n];
    // for i in 0..n {
        // ranks[suffix_array[i]] = i;
    // }

    // let mut l = 0usize;
    // let mut answer = 0usize;

    // for i in 0..n {
        // let sa_index = ranks[i];
        // if sa_index == 0 {
            // continue;
        // }

        // let neighbor = suffix_array[sa_index - 1];

        // while i + l < n &&
            // neighbor + l < n &&
            // bytes[i + l] == bytes[neighbor + l] {
            // l += 1;
        // }

        // if l > answer {
            // answer = l;
        // }

        // if l > 0 {
            // l -= 1;
        // }
    // }


    // println!("{}", answer);
   
    // return
// }