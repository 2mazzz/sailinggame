/// One classical RK4 step of `y' = f(y)` over `dt`.
pub fn rk4<const N: usize>(y: [f64; N], dt: f64, f: impl Fn(&[f64; N]) -> [f64; N]) -> [f64; N] {
    let add = |a: &[f64; N], b: &[f64; N], s: f64| {
        let mut out = *a;
        for i in 0..N {
            out[i] += b[i] * s;
        }
        out
    };
    let k1 = f(&y);
    let k2 = f(&add(&y, &k1, dt / 2.0));
    let k3 = f(&add(&y, &k2, dt / 2.0));
    let k4 = f(&add(&y, &k3, dt));
    let mut out = y;
    for i in 0..N {
        out[i] += dt / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
    }
    out
}
