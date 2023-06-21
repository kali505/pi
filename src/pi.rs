use itertools::Itertools;
use rug::{ops::Pow, Integer};
use std::{thread, thread::JoinHandle};

// log10(640320^3 / 24 / 72) = 14.181647463
// this is x * 1_000_000_000
const DIGITS_PER_ITER: u64 = 14_181_647_463;
const DIGITS_PER_ITER_TIMES: u64 = 1_000_000_000;

const THREADS: usize = 4;
const MAX_ITER_PER_THREAD: usize = 10_000;

fn calc_p(start: u32, len: usize) -> Vec<Integer> {
    if len == 0 {
        return Vec::new();
    }

    let mut a: Integer = 6 * Integer::from(start) - 5;
    let mut b: Integer = 2 * Integer::from(start) - 1;
    let mut c: Integer = 6 * Integer::from(start) - 1;
    let mut ret: Vec<Integer> = vec![a.clone() * &b * &c];

    for _i in 1..len {
        a += 6;
        b += 2;
        c += 6;

        // x = last + 6ab
        let mut x = a.clone();
        x *= &b;
        x *= &c;
        ret.push(x);
    }

    if start == 0 {
        ret[0] = Integer::from(1);
    }

    return ret;
}

fn calc_q(start: u32, len: usize) -> Vec<Integer> {
    if len == 0 {
        return Vec::new();
    }

    let mut ret = vec![Integer::from(start).pow(3)];

    let mut k: Integer = Integer::from(start) + 1;
    //ret[_i] = k^3
    for _i in 1..len {
        // x = last + 3i(i-1) + 1
        let mut x = k.clone() - 1;
        x *= &k;
        x *= 3;
        x += 1;
        x += ret.last().unwrap();

        ret.push(x);
        k += 1;
    }

    // ret[i] *= 640320 ^ 3 / 24
    let a = 640320_u64.pow(2) * 26680;
    for x in ret.iter_mut() {
        *x *= a;
    }

    if start == 0 {
        ret[0] = Integer::from(1);
    }

    return ret;
}

fn calc_t(start: u32, len: usize, v_p: &Vec<Integer>) -> Vec<Integer> {
    if len == 0 {
        return Vec::new();
    }

    let mut ret = vec![Integer::from(13591409)];
    ret[0] += Integer::from(545140134) * start;

    // ret[i] = a[i]
    for _i in 1..len {
        let mut x = ret.last().unwrap().clone();
        x += 545140134;
        ret.push(x);
    }
    //println!("{:?} {:?}", v_p, ret);

    // T[i] = a[i] * p[i]
    let is_start_odd = start & 1 == 1;
    for i in 0..len {
        ret[i] *= &v_p[i];
        // i & 1 != 1(false), start is odd => odd
        // i & 1 == 1(true), start is not odd => odd
        if (i & 1 == 1) != is_start_odd {
            ret[i] *= -1;
        }
    }
    //println!("{:?} {:?}", v_p, ret);

    return ret;
}

fn calc_sum(v_p: &mut [Integer], v_q: &mut [Integer], v_t: &mut [Integer]) {
    let mut iter_t = v_t.iter_mut().rev();
    let mut iter_p = v_p.iter_mut().rev();
    let mut iter_q = v_q.iter_mut().rev();

    let mut r_t = iter_t.next().unwrap();
    let mut r_p = iter_p.next().unwrap();
    let mut r_q = iter_q.next().unwrap();

    while let Some(this_t) = iter_t.next()
            && let Some(this_p)= iter_p.next()
            && let Some(this_q) = iter_q.next()  {
        *this_t *= &*r_q;
        *this_t += &*r_t * &*this_p;

        *this_p *= &*r_p;
        *this_q *= &*r_q;

        *r_t = Integer::ZERO;
        *r_p = Integer::ZERO;
        *r_q = Integer::ZERO;

        r_t = this_t;
        r_p = this_p;
        r_q = this_q;
    }
}

fn calc_sum2(v_p: Vec<&mut Integer>, v_q: Vec<&mut Integer>, v_t: Vec<&mut Integer>) {
    let mut iter_t = v_t.into_iter().rev();
    let mut iter_p = v_p.into_iter().rev();
    let mut iter_q = v_q.into_iter().rev();

    let mut r_t = iter_t.next().unwrap();
    let mut r_p = iter_p.next().unwrap();
    let mut r_q = iter_q.next().unwrap();

    while let Some(this_t) = iter_t.next()
            && let Some(this_p)= iter_p.next()
            && let Some(this_q) = iter_q.next()  {
        *this_t *= &*r_q;
        *this_t += &*r_t * &*this_p;

        *this_p *= &*r_p;
        *this_q *= &*r_q;

        *r_t = Integer::ZERO;
        *r_p = Integer::ZERO;
        *r_q = Integer::ZERO;

        r_t = this_t;
        r_p = this_p;
        r_q = this_q;
    }
}

pub struct PiCalc {
    // summed p, q, t
    p: Integer,
    q: Integer,
    t: Integer,
    // p,q,t start index
    pqt_start: u32,
    current_len: u32,
}

impl PiCalc {
    pub fn new() -> Self {
        Self {
            p: Integer::ZERO,
            q: Integer::ZERO,
            t: Integer::ZERO,
            pqt_start: 0,
            current_len: 0,
        }
    }

    pub fn get_pi(&mut self, len: u32) -> Integer {
        let s = Self::start_s_calc(len);

        if len == 0 {
            return Integer::ZERO;
        } else if len <= self.current_len {
            let pi = s.join().unwrap() * 426880 * &self.q / &self.t;
            return pi;
        }

        self.pre_calc(len);
        let pi = s.join().unwrap() * 426880 * &self.q / &self.t;
        return pi;
    }

    pub fn pre_calc(&mut self, len: u32) {
        if len == 0 || len <= self.current_len {
            return;
        }

        let need_iters: usize =
            ((len - self.current_len) as u64 * DIGITS_PER_ITER_TIMES / DIGITS_PER_ITER + 1)
                .try_into()
                .unwrap();
        let mut thread_pool = scoped_threadpool::Pool::new(THREADS.try_into().unwrap());

        let v_sum_len = if need_iters % MAX_ITER_PER_THREAD == 0 {
            need_iters / MAX_ITER_PER_THREAD
        } else {
            need_iters / MAX_ITER_PER_THREAD + 1
        };
        let mut v_sum_p = vec![Integer::ZERO; v_sum_len];
        let mut v_sum_q = vec![Integer::ZERO; v_sum_len];
        let mut v_sum_t = vec![Integer::ZERO; v_sum_len];

        thread_pool.scoped(|scope| {
            let mut start_i = self.pqt_start;
            for ((sp, sq), st) in v_sum_p
                .iter_mut()
                .zip(v_sum_q.iter_mut())
                .zip(v_sum_t.iter_mut())
            {
                scope.execute(move || {
                    let mut v_p = calc_p(start_i, MAX_ITER_PER_THREAD);
                    let mut v_q = calc_q(start_i, MAX_ITER_PER_THREAD);
                    let mut v_t = calc_t(start_i, MAX_ITER_PER_THREAD, &v_p);
                    calc_sum(&mut v_p, &mut v_q, &mut v_t);

                    *sp = v_p[0].clone();
                    *sq = v_q[0].clone();
                    *st = v_t[0].clone();
                });
                start_i += MAX_ITER_PER_THREAD as u32;
            }
        });

        let chunk_sz = if MAX_ITER_PER_THREAD % 0x100 == 0 {
            MAX_ITER_PER_THREAD / 0x100
        } else {
            MAX_ITER_PER_THREAD / 0x100 + 1
        };
        let mut step = 1;

        while v_sum_len > step {
            thread_pool.scoped(|scope| {
                let iter_cp = v_sum_p.iter_mut().step_by(step).chunks(chunk_sz);
                let iter_cq = v_sum_q.iter_mut().step_by(step).chunks(chunk_sz);
                let iter_ct = v_sum_t.iter_mut().step_by(step).chunks(chunk_sz);

                for (chunk_cp, (chunk_cq, chunk_ct)) in iter_cp
                    .into_iter()
                    .zip(iter_cq.into_iter().zip(iter_ct.into_iter()))
                {
                    let cp: Vec<&mut Integer> = chunk_cp.collect();
                    let cq: Vec<&mut Integer> = chunk_cq.collect();
                    let ct: Vec<&mut Integer> = chunk_ct.collect();

                    scope.execute(move || {
                        calc_sum2(cp, cq, ct);
                    });
                }
                step *= chunk_sz;
            });
        }

        self.p = v_sum_p[0].clone();
        self.q = v_sum_q[0].clone();
        self.t = v_sum_t[0].clone();
    }

    fn start_s_calc(len: u32) -> JoinHandle<Integer> {
        thread::spawn(move || {
            // s = sqrt(10005 * 10^(2*len))
            let mut s = Integer::from(5).pow(2 * len);
            s <<= 2 * len;
            s *= 10005;

            return s.sqrt();
        })
    }
}

#[cfg(test)]
mod tests {
    use rug::Integer;

    use super::{calc_p, calc_q, calc_t};

    #[test]
    fn test_calc_p() {
        let should_be = vec![1, 5, 231, 1105, 3059, 6525, 11935];
        let result = calc_p(0, 7);

        assert_eq!(should_be, result);
    }

    #[test]
    fn test_calc_q() {
        let should_be = vec![
            1,
            10939058860032000_u64,
            87512470880256000,
            295354589220864000,
            700099767042048000,
            1367382357504000000,
        ];
        let result = calc_q(0, 6);

        assert_eq!(should_be, result);
    }

    #[test]
    fn test_calc_t() {
        let should_be = vec![
            13591409_i64,
            -2793657715,
            254994357387,
            -1822158051155,
            6711910799755,
            -17873880815475,
        ];
        let result = calc_t(
            0,
            6,
            &vec![
                Integer::from(1),
                Integer::from(5),
                Integer::from(231),
                Integer::from(1105),
                Integer::from(3059),
                Integer::from(6525),
            ],
        );

        assert_eq!(should_be, result);
    }
}
