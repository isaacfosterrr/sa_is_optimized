
use std::vec;
use core::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};

use std::time::Instant;

pub trait Packed: Copy + PartialEq {
    fn is_s(self) -> bool;
    fn sym(self) -> usize;
    fn pack(sym: usize, is_s: bool) -> Self;
	
}

impl Packed for u16 {
	
    #[inline(always)] fn is_s(self) -> bool { 
		self & 0x8000 != 0 
		}
    #[inline(always)] fn sym(self) -> usize { 
		(self & 0x7FFF) as usize 
		}
    #[inline(always)] fn pack(s: usize, is_s: bool) -> Self {
        debug_assert!(s <= 0x7FFF);
        (s as u16) | if is_s { 0x8000 } else { 0 }
    }
}

impl Packed for u32 {
    #[inline(always)] fn is_s(self) -> bool { self & 0x8000_0000 != 0 }
    #[inline(always)] fn sym(self) -> usize { (self & 0x7FFF_FFFF) as usize }
    #[inline(always)] fn pack(s: usize, is_s: bool) -> Self {
        debug_assert!(s <= 0x7FFF_FFFF);
        (s as u32) | if is_s { 0x8000_0000 } else { 0 }
    }
}
#[inline(always)] pub fn is_lms_p<P: Packed>(packed: &[P], i: usize) -> bool {
		i > 0 && packed[i].is_s() && !packed[i - 1].is_s()
	}


pub const S: u8 = 0;
pub const L: u8 = 1;

const S_BIT: u16 = 0x8000;      // flag set = S-type, clear = L-type
const SYM_MASK: u16 = 0x7FFF;

// traits needed for using both usize and u8 inputs in functions
// as_index is a cast to usize for indexing
pub trait Symbol: Ord + Copy {
    fn as_index(&self) -> usize;
    fn display(&self) -> String;
    fn sentinel() -> Self;
}

impl Symbol for u8 {
    fn as_index(&self) -> usize { *self as usize }
    fn display(&self) -> String {
        if self.is_ascii() { (*self as char).to_string() }
        else { ".".to_string() }
    }
    fn sentinel() -> Self { 0 }
}

impl Symbol for u32 {
    fn as_index(&self) -> usize { *self as usize }
    fn display(&self) -> String {
        match char::from_u32(*self) {
			Some(c) if c.is_ascii() => c.to_string(),
			_ => self.to_string(),
		}
        
    }
    fn sentinel() -> Self { 0 }
}

pub  fn  print_aligned<T: Symbol>(bytes: &[T], type_map: &[u8], sa: &[usize]) {
    assert_eq!(bytes.len(), type_map.len(), "Vectors must be the same length");

    // Print indices
    print!("Index: ");
    for i in 0..bytes.len() {
        print!("{:<3} ", i);
    }
    println!();

    // Print types
    print!("Type : ");
    for t in type_map {
        let tipe: char = match t {
            &S => 'S',
            &L => 'L',
            _ => '?',
        };
        print!("{:<3} ", tipe);
    }
    println!();

    // Print characters safely
    print!("Char : ");
    for &b in bytes {
        print!("{:<3} ", b.display());
    }
    println!();
    // Print SA
    print!("rank : ");
    for &b in sa {
        if b != usize::MAX { 
            print!("{:<3} ", b);
        } else { 
            print!("{:<3} ", '.');
        }
        
    }

    println!();
}


pub fn is_lms(type_map: &[u8], i: usize) -> bool {
    // checks if suffix at index i is lms (left most s-type symbol)
    if i < 1 || i >= type_map.len() {
		return false;
	}
    if type_map[i] == S && type_map[i - 1] == L {
        true
    } else {
        false
    }
}

pub  fn  find_types<T: Symbol>(bytes: &[T]) -> Vec<u8> {
    // types - s or l, index i is s-type if string[i] < string[i+1]
    // and vice versa for l
    if bytes.is_empty() { return vec![]; }
    let mut type_map: Vec<u8> = vec![0; bytes.len()];

    for i in (0..bytes.len() - 1).rev(){
       if bytes[i] < bytes[i + 1] {
            type_map[i] = S;
        } else if bytes[i] == bytes[i + 1] {
            type_map[i] = type_map[i + 1];
        } else {
            type_map[i] = L;
        }
    }
    type_map[bytes.len() - 1] = S;
    type_map
}

// pub fn find_lms(type_map: &[u8]) -> Vec<u32>{
    // let mut lms_positions = Vec::new();
    // for i in 1..type_map.len() {
        // if is_lms(type_map, i) {
            // lms_positions.push(i as u32);
        // }
    // }
    // lms_positions
// }

pub fn find_lms<P: Packed>(packed: &[P]) -> Vec<u32> {
	let mut lms_positions = vec![0u32; packed.len() / 2 + 1];
	let mut k = 0;
	for i in 1..packed.len() {
		lms_positions[k] = i as u32;
		k += (packed[i].is_s() & !packed[i - 1].is_s()) as usize;
	}
	lms_positions.truncate(k);
	lms_positions
}

pub fn get_frequencies<T: Symbol>(bytes: &[T]) -> Vec<u32> {
    // finds how many of each character there is in string
    // by using simple array, since we know size of alphabet
    let mut max_symbol = 0;
    for &b in bytes {
        let v = b.as_index();
        if v > max_symbol {
            max_symbol = v;
        }
    }

    let size = (max_symbol + 1).max(256);
    let mut freq = vec![0; size];

    for &b in bytes {
        freq[b.as_index()] += 1;
    }

    freq
}
pub fn get_heads_tails(freq: &[u32]) -> (Vec<u32>, Vec<u32>) {
    // finds start and end for each kind of character
    // so if theres string like "bbbbbbba"
    // start for b is 0, end is 6
    // start for a is 7, end is 7
    let mut cur_start = 0;
    let mut heads = vec![0u32; freq.len()];
    let mut tails = vec![0u32; freq.len()];
    for i in 0..freq.len() {
        if freq[i] != 0 {
            heads[i] = cur_start;
            cur_start += freq[i];
            tails[i] = cur_start - 1;
        }
    }
    (heads, tails)
}

pub fn place_lms<P: Packed>(lms: &[u32], tails: &mut [u32], packed: &[P]) -> Vec<u32> {
    let mut sa = vec![u32::MAX; packed.len()];

    for i in (0..lms.len()).rev() {
        let pos = lms[i] as usize;
        let ch = packed[pos].sym();
        let bucket_end = tails[ch] as usize;
        sa[bucket_end] = lms[i];
        
        if tails[ch] > 0 {
            tails[ch] -= 1;	
        }
    }
    sa
}


pub fn induce_l<P: Packed>(packed: &[P], sa: &mut [u32], heads: &mut [u32]) {
	const D: usize = 32;
	let n = packed.len();
    for i in 0..packed.len() {
		if i + D < n {
			let nxt = sa[i + D];                      // u32 from sa, not from note
			if nxt != u32::MAX && nxt > 1 {
				unsafe {
					_mm_prefetch(
						packed.as_ptr().add((nxt - 1) as usize) as *const i8,
						_MM_HINT_T0,
					);
				}
			}
		}
        let index = sa[i];
        if index == u32::MAX || index == 0 { continue; }
        let v = packed[(index - 1) as usize];      
        if !v.is_s() {
            let ch = v.sym();
            let slot = heads[ch] as usize;
            sa[slot] = index - 1;
            heads[ch] += 1;
        
        }
    }
}

pub  fn  induce_s<P: Packed>(packed: &[P], sa: &mut [u32], tails: &mut [u32]) {
    // right to left scan
	const D: usize = 32;
	let n = packed.len();
    for i in (0..n).rev() {
		if i >= D {
			let nxt = sa[i - D];
			if nxt != u32::MAX && nxt > 0 {
				unsafe {
					_mm_prefetch(packed.as_ptr().add((nxt - 1) as usize) as *const i8, _MM_HINT_T0);
				}
			}
		}
		let index = sa[i];
		if index == u32::MAX || index == 0 { continue; }
		
		let v = packed[(index - 1) as usize];
		if v.is_s() {      // in the tails we have chars as indexes and its bucket end as values 
			let ch = v.sym();
			let bucket_end = tails[ch] as usize;
			sa[bucket_end] = index - 1;   
			tails[ch] = tails[ch].wrapping_sub(1);
        }
		
    }
}

pub fn is_lms_equal<T: Symbol>(bytes: &[T], type_map: &[u8], mut lms_one: usize, mut lms_two: usize) -> bool {
    // here lms is portion of string that starts with that lms and ends with next lms
    // whole string made of this lms substrings
    if bytes[lms_one] == bytes[lms_two] {
        lms_one += 1;
        lms_two += 1;
    } else {
        return false;
    }
    while lms_one < bytes.len() && lms_two < type_map.len() {
        if !is_lms(type_map, lms_one) && !is_lms(type_map, lms_two) {
            if bytes[lms_one] != bytes[lms_two] || type_map[lms_one] != type_map[lms_two] {
                return false;
            }
            lms_one += 1;
            lms_two += 1;
        } else if is_lms(type_map, lms_one) && is_lms(type_map, lms_two) {
            return true;
        } else {
            return false; // one ended before the other
        }
    }
    true
}
pub fn is_lms_equal_p<P: Packed>(packed: &[P], a: usize, b: usize) -> bool {
    if a == b { return true; }
    let n = packed.len();
    let mut k = 0;
    loop {
        if a + k >= n || b + k >= n { return false; }   // ran into the sentinel end
        if packed[a + k] != packed[b + k] { return false; } // symbol or type differs
        if k > 0 && (is_lms_p(packed, a + k) || is_lms_p(packed, b + k)) {
            // both are LMS here (symbols and types matched), so both substrings end together
            return true;
        }
        k += 1;
    }
}

pub fn summarize<P: Packed>(guessed_sa: &[u32], packed: &[P]) -> (Vec<u32>, Vec<u32>, u32) {
    // assign names, which is just numbers starting with 1
    // those names are used to determine whether lms are sorted
    // if names unique - sorted, if not - possibly not sorted yet
    let mut slots: Vec<u64> = vec![u64::MAX; packed.len() / 2 + 1];
	
    let mut summary: Vec<u32> = Vec::new();
    let mut suffix_offsets: Vec<u32> = Vec::new();
    let mut last_lms: Option<usize> = None;
    let mut current_name = 1;
	
	const D: usize = 32;
    for k in 0..guessed_sa.len() {
        // prefetch the random element needed D iterations from now
        if k + D < guessed_sa.len() {
            let nxt = guessed_sa[k + D];
            if nxt != u32::MAX {
                unsafe {
                    let p = packed.as_ptr().add(nxt as usize) as *const i8;
                    _mm_prefetch(p, _MM_HINT_T0);
                }
            }
        }

        let entry = guessed_sa[k];
        if entry == u32::MAX { continue; }
        let i = entry as usize;
        if is_lms_p(packed, i) {
            if let Some(last) = last_lms {
                if !is_lms_equal_p(packed, i, last) {
                    current_name += 1;
                }
            }
            slots[i / 2] = ((current_name as u64) << 32) | i as u64;
            last_lms = Some(i);
        }
    }

    for &s in slots.iter() {
        if s != u64::MAX {
            summary.push((s >> 32) as u32);
            suffix_offsets.push(s as u32);
        }
    }


    (summary, suffix_offsets, current_name + 1)
}

// pub fn summarize<T: Symbol>(guessed_sa: &[usize], bytes: &[T], type_map: &[u8]) -> (Vec<usize>, Vec<usize>, usize) {
    // // assign names, which is just numbers starting with 1
    // // those names are used to determine whether lms are sorted
    // // if names unique - sorted, if not - possibly not sorted yet
    // let mut intermidiate_summary: Vec<usize> = vec![usize::MAX; bytes.len()];
    // let mut summary: Vec<usize> = Vec::new();
    // let mut suffix_offsets: Vec<usize> = Vec::new();
    // let mut last_lms: Option<usize> = None;
    // let mut current_name = 1;

    // for i in guessed_sa {
        // if is_lms(type_map, *i) {   // here lms' got placed at index, which is same as their original index in bytes
            // if let Some(last) = last_lms {
                // if !is_lms_equal(bytes, type_map, *i, last) {
                    // current_name += 1;
                // }
            // }
            // intermidiate_summary[*i] = current_name;
            // last_lms = Some(*i);
        // }

    // }
    // for (index, &name) in intermidiate_summary.iter().enumerate() {
        // if name != usize::MAX { // in summary names go in order lms appear in original string
            // summary.push(name); // and suffix offset tells exactly where they are in original string
            // suffix_offsets.push(index);
        // }
    // }

    // (summary, suffix_offsets, current_name + 1)
// }

pub fn build_packed<T: Symbol, P: Packed>(bytes: &[T], type_map: &[u8]) -> Vec<P> {
    bytes
        .iter()
        .zip(type_map.iter())
        .map(|(&b, &t)| P::pack(b.as_index(), t != L))
        .collect()
}



pub fn induce_all<T: Symbol, P: Packed>(
    bytes: &[T],
    freq: &[u32],
	type_map: &[u8],
    depth: usize,
) -> Vec<u32> {
    let packed: Vec<P> = build_packed(bytes, type_map);
	let t = Instant::now();
    let (heads, tails) = get_heads_tails(freq);
    eprintln!(
        "get head tails depth={depth}, n={}, time={:.3}s",
        bytes.len(),
        t.elapsed().as_secs_f64()
    );
    
	let t = Instant::now();
	let lms = find_lms(&packed);
	eprintln!(
        "find_lms depth={depth}, n={}, time={:.3}s",
        bytes.len(),
        t.elapsed().as_secs_f64()
    );
	
	let t = Instant::now();
    let mut guessed_sa = place_lms(&lms, &mut tails.clone(), &packed);
    eprintln!(
        "place_lms 1 depth={depth}, n={}, time={:.3}s",
        bytes.len(),
        t.elapsed().as_secs_f64()
    );
    

    let t = Instant::now();
    induce_l(&packed, &mut guessed_sa, &mut heads.clone());
    eprintln!(
        "induce_l 1 depth={depth}, n={}, time={:.3}s",
        bytes.len(),
        t.elapsed().as_secs_f64()
    );

    let t = Instant::now();
    induce_s(&packed, &mut guessed_sa, &mut tails.clone());
    eprintln!(
        "induce_s 1 depth={depth}, n={}, time={:.3}s",
        bytes.len(),
        t.elapsed().as_secs_f64()
    );


    let t = Instant::now();
	let n = packed.len();
    let (summary, suffix_offsets, name_count) = summarize(&guessed_sa[0..n], &packed);
    eprintln!(
        "summarize depth={depth}, n={}, time={:.3}s",
        bytes.len(),
        t.elapsed().as_secs_f64()
    );


    let sorted_lms: Vec<u32> = if (name_count - 1) as usize == summary.len() {
        guessed_sa
            .iter()
            .copied()
            .filter(|&i| i != u32::MAX && is_lms(type_map, i as usize))
            .collect()
    } else {
        let recursive_sa = build_sa_is(&summary, depth + 1);
        recursive_sa
            .iter()
            .map(|&i| suffix_offsets[i as usize])
            .collect()
    };

	let t = Instant::now();
    let mut final_sa = place_lms(&sorted_lms, &mut tails.clone(), &packed);
    eprintln!(
        "place_lms 2 depth={depth}, n={}, time={:.3}s",
        bytes.len(),
        t.elapsed().as_secs_f64()
    );
    

    let t = Instant::now();
    induce_l(&packed, &mut final_sa, &mut heads.clone());
    eprintln!(
        "induce_l 2 depth={depth}, n={}, time={:.3}s",
        bytes.len(),
        t.elapsed().as_secs_f64()
    );

    let t = Instant::now();
    induce_s(&packed, &mut final_sa, &mut tails.clone());
    eprintln!(
        "induce_s 2 depth={depth}, n={}, time={:.3}s",
        bytes.len(),
        t.elapsed().as_secs_f64()
    );

    final_sa
}



pub fn build_sa_is<T: Symbol>(bytes: &[T], depth: usize) -> Vec<u32> {
    let mut bytes_with_sentinel = bytes.to_vec();
    bytes_with_sentinel.push(T::sentinel());
    let bytes = &bytes_with_sentinel;
	let t = Instant::now();
    let freq = get_frequencies(bytes);
	eprintln!(
		"get_frequencies depth={depth}, n={}, first L-induce={:.3}s",
		bytes.len(),
		t.elapsed().as_secs_f64()
	);
	
	let t = Instant::now();
    let type_map = &find_types(bytes); 
	eprintln!(
		"type_map depth={depth}, n={}, first L-induce={:.3}s",
		bytes.len(),
		t.elapsed().as_secs_f64()
	);
	
    let final_sa = if freq.len() <= 0x7FFF {
		let t = Instant::now();
		let r = induce_all::<T, u16>(bytes, &freq, &type_map, depth);
		eprintln!("induce_all u16 depth={depth}, n={}, {:.3}s", bytes.len(), t.elapsed().as_secs_f64());
		r
	} else {
		let t = Instant::now();
		let r = induce_all::<T, u32>(bytes, &freq, &type_map, depth);
		eprintln!("induce_all u32 depth={depth}, n={}, {:.3}s", bytes.len(), t.elapsed().as_secs_f64());
		r
	};
	let t = Instant::now();
    let final_sa = final_sa[1..].to_vec();
	eprintln!(
		"final sa to vec depth={depth}, n={}, first L-induce={:.3}s",
		bytes.len(),
		t.elapsed().as_secs_f64()
	);
	final_sa
}
