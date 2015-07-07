use distribution::{Discrete, Distribution};
use random::Source;

/// A categorical distribution.
#[derive(Clone)]
pub struct Categorical {
    k: usize,
    p: Vec<f64>,
}

impl Categorical {
    /// Create a categorical distribution with success probability `p`.
    ///
    /// It should hold that `p[i] >= 0`, `p[i] <= 1`, and `sum(p) == 1`.
    #[inline]
    pub fn new(p: &[f64]) -> Categorical {
        should!(is_probability_vector(p), {
            const EPSILON: f64 = 1e-12;
            let mut in_unit = true;
            let mut sum = 0.0;
            for &p in p.iter() {
                if p < 0.0 || p > 1.0 {
                    in_unit = false;
                    break;
                }
                sum += p;
            }
            in_unit && (sum - 1.0).abs() <= EPSILON
        });
        Categorical { k: p.len(), p: p.to_vec() }
    }

    /// Return the number of categories.
    #[inline(always)]
    pub fn k(&self) -> usize { self.k }

    /// Return the event probabilities.
    #[inline(always)]
    pub fn p(&self) -> &[f64] { &self.p }
}

impl Distribution for Categorical {
    type Value = usize;

    fn mean(&self) -> f64 {
        self.p.iter().enumerate().fold(0.0, |sum, (i, p)| sum + i as f64 * p)
    }

    fn var(&self) -> f64 {
        let mean = self.mean();
        self.p.iter().enumerate().fold(0.0, |sum, (i, p)| sum + (i as f64 - mean).powi(2) * p)
    }

    fn skewness(&self) -> f64 {
        let (mean, var) = (self.mean(), self.var());
        let skew = self.p.iter().enumerate()
                                .fold(0.0, |sum, (i, p)| sum + (i as f64 - mean).powi(3) * p);
        skew / (var * var.sqrt())
    }

    fn kurtosis(&self) -> f64 {
        let (mean, var) = (self.mean(), self.var());
        let kurt = self.p.iter().enumerate()
                                .fold(0.0, |sum, (i, p)| sum + (i as f64 - mean).powi(4) * p);
        kurt / var.powi(2) - 3.0
    }

    fn median(&self) -> f64 {
        if self.p[0] > 0.5 {
            return 0.0;
        } else if self.p[0] == 0.5 {
            return 0.5;
        }
        let mut sum = 0.0;
        for i in 0..self.k {
            sum += self.p[i];
            if sum == 0.5 {
                return (2 * i - 1) as f64 / 2.0;
            } else if sum > 0.5 {
                return i as f64;
            }
        }
        unreachable!()
    }

    fn modes(&self) -> Vec<usize> {
        let mut modes = Vec::new();
        let mut max = 0.0;
        for (i, &p) in self.p.iter().enumerate() {
            if p == max {
                modes.push(i);
            }
            if p > max {
                max = p;
                modes = vec![i];
            }
        }
        modes
    }

    #[inline]
    fn entropy(&self) -> f64 {
        -self.p.iter().fold(0.0, |sum, p| sum + p * p.ln())
    }

    #[inline]
    fn cdf(&self, x: f64) -> f64 {
        if x < 0.0 {
            return 0.0;
        }
        let x = x as usize;
        if x >= self.k - 1 {
            1.0
        } else {
            self.p.iter().take(x + 1).fold(0.0, |a, b| a + b)
        }
    }

    fn inv_cdf(&self, p: f64) -> usize {
        should!(0.0 <= p && p <= 1.0);
        if p == 0.0 {
            return self.p.iter().position(|&p| p > 0.0).unwrap();
        }
        let mut sum = 0.0;
        for i in 0..self.k {
            sum += self.p[i];
            if sum >= p || sum == 1.0 {
                return i;
            }
        }
        self.k - 1
    }

    #[inline]
    fn pmf(&self, x: usize) -> f64 {
        self.p[x]
    }

    #[inline]
    fn sample<S>(&self, source: &mut S) -> usize where S: Source {
        self.inv_cdf(source.read::<f64>())
    }
}

impl Discrete for Categorical {
}

#[cfg(test)]
mod tests {
    use prelude::*;

    macro_rules! new(
        (equal $k:expr) => { Categorical::new(&[1.0 / $k as f64; $k]) };
        ($p:expr) => { Categorical::new(&$p); }
    );

    #[test]
    fn mean() {
        assert_eq!(new!(equal 3).mean(), 1.0);
        assert_eq!(new!([0.3, 0.3, 0.4]).mean(), 1.1);
        assert_eq!(new!([1.0 / 6.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0]).mean(), 1.5);
    }

    #[test]
    fn var() {
        assert_eq!(new!(equal 3).var(), 2.0 / 3.0);
        assert_eq!(new!([1.0 / 6.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0]).var(), 11.0 / 12.0);
    }

    #[test]
    fn sd() {
        assert_eq!(new!(equal 2).sd(), 0.5);
        assert_eq!(new!([1.0 / 6.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0]).sd(), 0.9574271077563381);
    }

    #[test]
    fn skewness() {
        assert_eq!(new!(equal 6).skewness(), 0.0);
        assert_eq!(new!([1.0 / 6.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0]).skewness(), 0.0);
        assert_eq!(new!([0.1, 0.2, 0.3, 0.4]).skewness(), -0.6);
    }

    #[test]
    fn kurtosis() {
        assert_eq!(new!(equal 2).kurtosis(), -2.0);
        assert_eq!(new!([0.1, 0.2, 0.3, 0.4]).kurtosis(), -0.7999999999999998);
    }

    #[test]
    fn median() {
        assert_eq!(new!([0.6, 0.2, 0.2]).median(), 0.0);
        assert_eq!(new!(equal 2).median(), 0.5);
        assert_eq!(new!([0.1, 0.2, 0.3, 0.4]).median(), 2.0);
        assert_eq!(new!([1.0 / 6.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0]).median(), 0.5);
    }

    #[test]
    fn modes() {
        assert_eq!(new!([0.6, 0.2, 0.2]).modes(), vec![0]);
        assert_eq!(new!(equal 2).modes(), vec![0, 1]);
        assert_eq!(new!(equal 3).modes(), vec![0, 1, 2]);
        assert_eq!(new!([0.4, 0.2, 0.4]).modes(), vec![0, 2]);
        assert_eq!(new!([1.0 / 6.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0]).modes(), vec![1, 2]);
    }

    #[test]
    fn entropy() {
        use std::f64::consts::LN_2;
        assert_eq!(new!(equal 2).entropy(), LN_2);
        assert_eq!(new!([0.1, 0.2, 0.3, 0.4]).entropy(), 1.2798542258336676);
    }

    #[test]
    fn pmf() {
        let p = [0.0, 0.75, 0.25, 0.0];
        let d1 = new!(p);
        assert_eq!(&(0..4).map(|x| d1.pmf(x)).collect::<Vec<_>>(), &p.to_vec());

        let d2 = new!(equal 3);
        assert_eq!(&(0..3).map(|x| d2.pmf(x)).collect::<Vec<_>>(), &vec![1.0 / 3.0; 3])
    }

    #[test]
    fn cdf() {
        let d = new!([0.0, 0.75, 0.25, 0.0]);
        let p = vec![0.0, 0.0, 0.75, 1.0, 1.0];

        let x = (-1..4).map(|x| d.cdf(x as f64)).collect::<Vec<_>>();
        assert_eq!(&x, &p);

        let x = (-1..4).map(|x| d.cdf(x as f64 + 0.5)).collect::<Vec<_>>();
        assert_eq!(&x, &p);

        let d = new!(equal 3);
        let p = vec![0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0];

        let x = (-1..3).map(|x| d.cdf(x as f64)).collect::<Vec<_>>();
        assert_eq!(&x, &p);

        let x = (-1..3).map(|x| d.cdf(x as f64 + 0.5)).collect::<Vec<_>>();
        assert_eq!(&x, &p);
    }

    #[test]
    fn inv_cdf() {
        let d1 = new!([0.0, 0.75, 0.25, 0.0]);
        let p1 = vec![0.0, 0.75, 0.7500001, 1.0];
        assert_eq!(&p1.iter().map(|&p| d1.inv_cdf(p)).collect::<Vec<_>>(), &vec![1, 1, 2, 2]);

        let d2 = new!(equal 3);
        let p2 = vec![0.0, 0.5, 0.75, 1.0];
        assert_eq!(&p2.iter().map(|&p| d2.inv_cdf(p)).collect::<Vec<_>>(), &vec![0, 1, 2, 2]);

    }

    #[test]
    fn sample() {
        let mut source = random::default();
        let sum = Independent(&new!([0.0, 0.5, 0.5]), &mut source).take(100).fold(0, |a, b| a + b);
        assert!(100 <= sum && sum <= 200);
    }
}
