use probability::prelude::*;
use test::{Bencher, black_box};

#[bench]
fn cdf(bencher: &mut Bencher) {
    let binom = Binomial::new(100_000, 0.845);
    let x = Sampler(&binom, &mut generator()).take(1000).collect::<Vec<_>>();

    bencher.iter(|| black_box(x.iter().map(|&x| binom.cdf(x)).collect::<Vec<_>>()));
}

#[bench]
fn inv_cdf(bencher: &mut Bencher) {
    let binom = Binomial::new(100_000, 0.845);
    let uniform = Uniform::new(0.0, 1.0);
    let p = Sampler(&uniform, &mut generator()).take(1000).collect::<Vec<_>>();

    bencher.iter(|| black_box(p.iter().map(|&p| binom.inv_cdf(p)).collect::<Vec<_>>()));
}

#[bench]
fn pdf(bencher: &mut Bencher) {
    let binom = Binomial::new(100_000, 0.845);
    let x = Sampler(&binom, &mut generator()).take(1000).collect::<Vec<_>>();

    bencher.iter(|| black_box(x.iter().map(|&x| binom.pdf(x)).collect::<Vec<_>>()));
}