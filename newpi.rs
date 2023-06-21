/*thread_pool.scoped(|scope| {
            for ((cp, cq), ct) in v_sum_p
                .chunks_mut(chunk_sz)
                .zip(v_sum_q.chunks_mut(chunk_sz))
                .zip(v_sum_t.chunks_mut(chunk_sz))
            {
                scope.execute(|| {
                    calc_sum(cp, cq, ct);
                });
            }
        });

        let mut i = 0;
        while i * chunk_sz < v_sum_len {
            v_sum_p.swap(i, i * chunk_sz);
            v_sum_q.swap(i, i * chunk_sz);
            v_sum_t.swap(i, i * chunk_sz);

            i += 1;
        }
        calc_sum(&mut v_sum_p[0..i], &mut v_sum_q[0..i], &mut v_sum_t[0..i]);

*/
