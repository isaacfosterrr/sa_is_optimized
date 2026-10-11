
use sa_linear::*;

fn naive_sa(s: &[u8]) -> Vec<usize> {
    let mut sa: Vec<usize> = (0..s.len()).collect();
    sa.sort_by(|&i, &j| s[i..].cmp(&s[j..]));
    sa
}

#[test]
fn sa_matches_naive() {
    for seed in 0..500u64 {
        let mut x = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let n = 1 + (seed % 60) as usize;
        let bytes: Vec<u8> = (0..n).map(|_| {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            1 + ((x >> 33) % 3) as u8
        }).collect();
        let mut expected: Vec<u32> = (0..n as u32).collect();
        expected.sort_by(|&a, &b| bytes[a as usize..].cmp(&bytes[b as usize..]));
        assert_eq!(build_sa_is(&bytes, 0), expected, "{bytes:?}");
    }
}

//#[test]
fn lms_equal_matches_old() {
    for seed in 0..2000u64 {
        let mut x = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let n = 5 + (seed % 40) as usize;
        let mut bytes: Vec<u8> = (0..n).map(|_| {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            1 + ((x >> 33) % 3) as u8
        }).collect();
        bytes.push(0); // sentinel
        let type_map = find_types(&bytes);
        let packed: Vec<u16> = build_packed(&bytes, &type_map);
        for i in 0..bytes.len() {
            assert_eq!(is_lms(&type_map, i), is_lms_p(&packed, i), "is_lms i={i} {bytes:?}");
            for j in 0..bytes.len() {
                if is_lms(&type_map, i) && is_lms(&type_map, j) {
                    assert_eq!(
                        is_lms_equal(&bytes, &type_map, i, j),
                        is_lms_equal_p(&packed, i, j),
                        "equal i={i} j={j} {bytes:?}"
                    );
                }
            }
        }
    }
} 

#[test]
fn test_sa_is() {
    let mut seed = 123456789u64;

    fn rng(seed: &mut u64) -> u64 {
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        *seed >> 33
    }

    for test_id in 0..1000 {
        let n = if test_id < 990 {
            (rng(&mut seed) % 2000 + 1) as usize
        } else {
            (rng(&mut seed) % 200_000 + 1) as usize
        };

        let k = [2u64, 3, 4, 26][(rng(&mut seed) % 4) as usize];

        let s: Vec<u8> = (0..n)
            .map(|_| b'a' + (rng(&mut seed) % k) as u8)
            .collect();

        let text: Vec<u32> = s
            .iter()
            .map(|&c| (c - b'a' + 1) as u32)
            .collect();

        let sa_fast = build_sa_is(&text, 0);
        let sa_u8 = build_sa_is(&s, 0);

        assert_eq!(sa_u8, sa_fast);

        // Validate length.
        if sa_fast.len() != n {
            println!("FAIL length mismatch");
            println!("test_id: {}", test_id);
            println!("n: {}", n);
            println!("sa_fast.len(): {}", sa_fast.len());
            println!("string: {:?}", String::from_utf8_lossy(&s));
            panic!();
        }

        // Validate bounds.
        if let Some(&bad) = sa_fast.iter().find(|&&x| x as usize >= n) {
            println!("FAIL out-of-bounds index");
            println!("test_id: {}", test_id);
            println!("n: {}", n);
            println!("bad value: {}", bad);
            println!("string: {:?}", String::from_utf8_lossy(&s));
            println!(
                "sa_fast (first 50): {:?}",
                &sa_fast[..sa_fast.len().min(50)]
            );
            panic!();
        }

        // Check duplicates and missing indices.
        let mut seen = vec![false; n];

        for &x in &sa_fast {
            if seen[x as usize] {
                println!("FAIL duplicate index");
                println!("test_id: {}", test_id);
                println!("duplicate: {}", x);
                println!("string: {:?}", String::from_utf8_lossy(&s));
                panic!();
            }

            seen[x as usize] = true;
        }

        if seen.iter().any(|&x| !x) {
            println!("FAIL missing index");
            println!("test_id: {}", test_id);
            println!("string: {:?}", String::from_utf8_lossy(&s));
            panic!();
        }

        // Compare against the naive implementation for small inputs.
        if n <= 2000 {
            let sa_naive = naive_sa(&s);
            let sa_fast_usize: Vec<usize> =
                sa_fast.iter().map(|&x| x as usize).collect();

            if sa_fast_usize != sa_naive {
                println!("FAIL mismatch");
                println!("test_id: {}", test_id);
                println!("n: {}", n);
                println!("string: {:?}", String::from_utf8_lossy(&s));
                println!("fast:  {:?}", sa_fast);
                println!("naive: {:?}", sa_naive);

                for i in 0..n {
                    if sa_fast[i] as usize != sa_naive[i] {
                        println!("first diff at position {}", i);
                        println!("fast[{}] = {}", i, sa_fast[i]);
                        println!("naive[{}] = {}", i, sa_naive[i]);
                        break;
                    }
                }

                panic!();
            }
        } else {
            for w in sa_fast.windows(2) {
                assert!(
                    s[w[0] as usize..] < s[w[1] as usize..],
                    "order violated"
                );
            }
        }

        if test_id % 10 == 0 {
            println!("Passed test {}", test_id);
        }
    }

    println!("All tests passed!");
}

#[test]
fn test_edge_cases() {
    let mut cases: Vec<Vec<u8>> = vec![
        b"a".to_vec(),
        b"aa".to_vec(),
        b"ab".to_vec(),
        b"ba".to_vec(),
        vec![b'a'; 1000],
        b"ab".repeat(500),
        b"abc".repeat(300),
    ];

    // Short random block repeated many times.
    let block: Vec<u8> = (0..7)
        .map(|i| b'a' + (i * 5 % 3) as u8)
        .collect();

    cases.push(block.repeat(200));

    for s in cases {
        assert_eq!(
            build_sa_is(&s, 0)
                .iter()
                .map(|&x| x as usize)
                .collect::<Vec<_>>(),
            naive_sa(&s)
        );
    }
}

use sa_linear::{find_lms, Packed, L};

fn pack_types(type_map: &[u8]) -> Vec<u16> {
    type_map.iter().map(|&t| <u16 as Packed>::pack(0, t != L)).collect()
}

#[test]
fn test_find_lms() {
    let type_map = vec![L, S, L, L, S, L, L, S];
    assert_eq!(find_lms(&pack_types(&type_map)), vec![1, 4, 7]);
}

#[test]
fn test_find_lms_complex() {
    let type_map = vec![L, S, L, L, S, L, L, S, L, L, L, S];
    assert_eq!(find_lms(&pack_types(&type_map)), vec![1, 4, 7, 11]);
}

#[test]
fn test_frequencies_complex() {
    let freq = get_frequencies(b"mississippi");

    assert_eq!(freq[b'i' as usize], 4);
    assert_eq!(freq[b's' as usize], 4);
    assert_eq!(freq[b'p' as usize], 2);
    assert_eq!(freq[b'm' as usize], 1);
}

#[test]
fn test_heads_tails() {
    let freq = get_frequencies(b"cabbage");
    let (heads, tails) = get_heads_tails(&freq);

    for i in 0..255 {
        if freq[i] != 0 && freq[i + 1] != 0 {
            assert!(tails[i] < heads[i + 1]);
        }
    }

    for i in 0..256 {
        if freq[i] != 0 {
            assert_eq!(tails[i] - heads[i] + 1, freq[i]);
        }
    }
}

#[test]
fn test_heads_tails_complex() {
    let freq = get_frequencies(b"mississippi");
    let (heads, tails) = get_heads_tails(&freq);

    for i in 0..255 {
        if freq[i] != 0 && freq[i + 1] != 0 {
            assert!(tails[i] < heads[i + 1]);
        }
    }

    for i in 0..256 {
        if freq[i] != 0 {
            assert_eq!(tails[i] - heads[i] + 1, freq[i]);
        }
    }
}

#[test]
fn test_final_sa() {
    let cases = vec!["cabbage", "banana", "aaaaaa", "abbc"];

    for s in cases {
        let bytes = s.as_bytes();

        let text: Vec<u32> = bytes
            .iter()
            .map(|&c| (c - b'a' + 1) as u32)
            .collect();

        let sa = build_sa_is(&text, 0);

        let mut seen = vec![false; bytes.len()];

        for &i in &sa {
            seen[i as usize] = true;
        }

        assert!(seen.iter().all(|&x| x), "not a permutation: {s}");

        for i in 1..sa.len() {
            assert!(
                bytes[sa[i - 1] as usize..] <= bytes[sa[i] as usize..],
                "not sorted: {s}"
            );
        }
    }
}

#[test]
fn test_final_sa_harder_cases() {
    let cases = vec![
        "mississippi",
        "abababab",
        "abcabcabc",
        "zzzyzzzy",
        "baabaabac",
        "aaabbbbababbbaaaacda",
    ];

    for s in cases {
        let bytes = s.as_bytes();

        let text: Vec<u32> = bytes
            .iter()
            .map(|&c| (c - b'a' + 1) as u32)
            .collect();

        let sa = build_sa_is(&text, 0);
        let naive = naive_sa(bytes);

        let sa_usize: Vec<usize> =
            sa.iter().map(|&x| x as usize).collect();

        assert_eq!(sa_usize, naive, "naive gives different result: {s}");

        let mut seen = vec![false; bytes.len()];

        for &i in &sa {
            seen[i as usize] = true;
        }

        assert!(seen.iter().all(|&x| x), "not a permutation: {s}");

        for i in 1..sa.len() {
            assert!(
                bytes[sa[i - 1] as usize..] <= bytes[sa[i] as usize..],
                "not sorted: {s}"
            );
        }
    }
}
