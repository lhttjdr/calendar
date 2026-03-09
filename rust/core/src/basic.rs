//! 基础工具：zip、zipWith、omap、ozip、ozipWith。

use std::collections::HashMap;

/// zip([a,b], [c,d]) => [(a,c), (b,d)]；长度取两者较短。
pub fn zip2<A: Clone, B: Clone>(a: &[A], b: &[B]) -> Vec<(A, B)> {
    a.iter().zip(b).map(|(x, y)| (x.clone(), y.clone())).collect()
}

/// zip([a,b], [c,d], [e,f]) => [(a,c,e), (b,d,f)]；长度取三者较短。
pub fn zip3<A: Clone, B: Clone, C: Clone>(a: &[A], b: &[B], c: &[C]) -> Vec<(A, B, C)> {
    a.iter()
        .zip(b)
        .zip(c)
        .map(|((x, y), z)| (x.clone(), y.clone(), z.clone()))
        .collect()
}

/// zipWith(f, [a,b], [c,d]) => [f(a,c), f(b,d)]
pub fn zip_with2<A: Clone, B: Clone, C, F: Fn(A, B) -> C>(a: &[A], b: &[B], f: F) -> Vec<C> {
    a.iter()
        .zip(b)
        .map(|(x, y)| f(x.clone(), y.clone()))
        .collect()
}

/// zipWith(f, [a,b], [c,d], [e,f]) => [f(a,c,e), f(b,d,f)]
pub fn zip_with3<A: Clone, B: Clone, C: Clone, D, F: Fn(A, B, C) -> D>(
    a: &[A],
    b: &[B],
    c: &[C],
    f: F,
) -> Vec<D> {
    zip3(a, b, c).into_iter().map(|(x, y, z)| f(x, y, z)).collect()
}

/// omap(m, f): 对 Map 的每个值应用 f
pub fn omap<K: Clone + std::hash::Hash + Eq, A, B, F: Fn(&A) -> B>(
    m: &HashMap<K, A>,
    f: F,
) -> HashMap<K, B> {
    m.iter().map(|(k, v)| (k.clone(), f(v))).collect()
}

/// ozip(m1, m2, m3): 多个 Map 按 key 对齐为 (k, [v1,v2,v3])
pub fn ozip<K: Clone + std::hash::Hash + Eq, A: Clone>(
    m1: &HashMap<K, A>,
    others: &[HashMap<K, A>],
) -> HashMap<K, Vec<A>> {
    let keys: Vec<K> = m1.keys().cloned().collect();
    keys.into_iter()
        .filter_map(|k| {
            let mut row = vec![];
            if let Some(v) = m1.get(&k) {
                row.push(v.clone());
            }
            for m in others {
                if let Some(v) = m.get(&k) {
                    row.push(v.clone());
                }
            }
            if row.is_empty() {
                None
            } else {
                Some((k, row))
            }
        })
        .collect()
}

/// ozipWith(zipper, m1, m2, m3): ozip 后对每个 key 的 value 序列应用 zipper
pub fn ozip_with<K: Clone + std::hash::Hash + Eq, A: Clone, B, F: Fn(Vec<A>) -> B>(
    m1: &HashMap<K, A>,
    others: &[HashMap<K, A>],
    zipper: F,
) -> HashMap<K, B> {
    omap(&ozip(m1, others), |row| zipper(row.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zip_2_returns_pairs() {
        let a = vec![1, 2];
        let b = vec![3, 4];
        assert_eq!(zip2(&a, &b), vec![(1, 3), (2, 4)]);
    }

    #[test]
    fn zip_3_returns_triples() {
        let a = vec![1, 2];
        let b = vec![3, 4];
        let c = vec![5, 6];
        assert_eq!(zip3(&a, &b, &c), vec![(1, 3, 5), (2, 4, 6)]);
    }

    #[test]
    fn zip_with_2_sum() {
        let a = vec![1, 2];
        let b = vec![3, 4];
        let out = zip_with2(&a, &b, |x, y| x + y);
        assert_eq!(out, vec![4, 6]);
    }

    #[test]
    fn zip_with_3_xy_plus_z() {
        let a = vec![1, 2];
        let b = vec![3, 4];
        let c = vec![5, 6];
        let out = zip_with3(&a, &b, &c, |x, y, z| x * y + z);
        assert_eq!(out, vec![8, 14]);
    }

    #[test]
    fn omap_double() {
        let mut m = HashMap::new();
        m.insert("a", 3);
        m.insert("b", 4);
        let out = omap(&m, |&v| v * 2);
        assert_eq!(out.get("a"), Some(&6));
        assert_eq!(out.get("b"), Some(&8));
    }

    #[test]
    fn ozip_three_maps() {
        let mut m1 = HashMap::new();
        m1.insert("a", 4);
        m1.insert("b", 7);
        let mut m2 = HashMap::new();
        m2.insert("a", 6);
        m2.insert("b", 1);
        let mut m3 = HashMap::new();
        m3.insert("a", 1);
        m3.insert("b", 7);
        let out = ozip(&m1, &[m2, m3]);
        assert_eq!(out.get("a"), Some(&vec![4, 6, 1]));
        assert_eq!(out.get("b"), Some(&vec![7, 1, 7]));
    }

    #[test]
    fn ozip_with_xy_plus_z() {
        let mut m1 = HashMap::new();
        m1.insert("a", 4);
        m1.insert("b", 7);
        let mut m2 = HashMap::new();
        m2.insert("a", 6);
        m2.insert("b", 1);
        let mut m3 = HashMap::new();
        m3.insert("a", 1);
        m3.insert("b", 7);
        let out = ozip_with(&m1, &[m2, m3], |row| {
            assert_eq!(row.len(), 3);
            row[0] * row[1] + row[2]
        });
        assert_eq!(out.get("a"), Some(&25));
        assert_eq!(out.get("b"), Some(&14));
    }
}
