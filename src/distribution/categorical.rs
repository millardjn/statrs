use std::f64;
use rand::Rng;
use rand::distributions::{Sample, IndependentSample};
use statistics::*;
use distribution::{Univariate, Discrete, Distribution};
use {Result, StatsError};

/// Implements the [Categorical](https://en.wikipedia.org/wiki/Categorical_distribution)
/// distribution, also known as the generalized Bernoulli or discrete distribution
///
/// # Examples
///
/// ```
/// use statrs::distribution::{Categorical, Discrete};
/// use statrs::statistics::Mode;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Categorical {
    norm_pmf: Vec<f64>,
    cdf: Vec<f64>,
}

impl Categorical {
    pub fn new(prob_mass: &[f64]) -> Result<Categorical> {
        if !is_valid_prob_mass(prob_mass) {
            Err(StatsError::BadParams)
        } else {
            // extract un-normalized cdf
            let mut cdf = vec![0.0; prob_mass.len()];
            cdf[0] = prob_mass[0];
            for i in 1..prob_mass.len() {
                unsafe {
                    let val = cdf.get_unchecked(i - 1) + prob_mass.get_unchecked(i);
                    let elem = cdf.get_unchecked_mut(i);
                    *elem = val;
                }
            }

            // extract normalized probability mass
            let sum = cdf[cdf.len() - 1];
            let mut norm_pmf = vec![0.0; prob_mass.len()];
            for i in 0..prob_mass.len() {
                unsafe {
                    let elem = norm_pmf.get_unchecked_mut(i);
                    *elem = prob_mass.get_unchecked(i) / sum;
                }
            }
            Ok(Categorical {
                norm_pmf: norm_pmf,
                cdf: cdf,
            })
        }
    }

    fn cdf_max(&self) -> f64 {
        *unsafe { self.cdf.get_unchecked(self.cdf.len() - 1) }
    }
}

impl Sample<f64> for Categorical {
    /// Generate a random sample from a categorical
    /// distribution using `r` as the source of randomness.
    /// Refer [here](#method.sample-1) for implementation details
    fn sample<R: Rng>(&mut self, r: &mut R) -> f64 {
        super::Distribution::sample(self, r)
    }
}

impl IndependentSample<f64> for Categorical {
    /// Generate a random independent sample from a categorical
    /// distribution using `r` as the source of randomness.
    /// Refer [here](#method.sample-1) for implementation details
    fn ind_sample<R: Rng>(&self, r: &mut R) -> f64 {
        super::Distribution::sample(self, r)
    }
}

impl Distribution<f64> for Categorical {
    /// Generate a random sample from the categorical distribution
    /// using `r` as the source of randomness
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate rand;
    /// # extern crate statrs;
    /// use rand::StdRng;
    /// use statrs::distribution::{Categorical, Distribution};
    ///
    /// # fn main() {
    /// let mut r = rand::StdRng::new().unwrap();
    /// let n = Categorical::new(&[1.0, 2.0, 3.0]).unwrap();
    /// print!("{}", n.sample::<StdRng>(&mut r));
    /// # }
    /// ```
    fn sample<R: Rng>(&self, r: &mut R) -> f64 {
        let draw = r.next_f64() * self.cdf_max();
        let mut idx = 0;

        if draw == 0.0 {
            // skip zero-probability categories
            let mut el = unsafe { self.cdf.get_unchecked(idx) };
            while *el == 0.0 {
                // don't need bounds checking because we do not allow
                // creating Categorical distributions with all 0.0 probs
                idx += 1;
                el = unsafe { self.cdf.get_unchecked(idx) }
            }
        }
        let mut el = unsafe { self.cdf.get_unchecked(idx) };
        while draw > *el {
            idx += 1;
            el = unsafe { self.cdf.get_unchecked(idx) };
        }
        return idx as f64;
    }
}

impl Univariate<u64, f64> for Categorical {
    /// Calculates the cumulative distribution function for the categorical
    /// distribution at `x`
    ///
    /// # Panics
    ///
    /// If `x < 0.0` or `x > k` where `k` is the number of categories
    /// (i.e. the length of the `prob_mass` slice passed to the constructor)
    ///
    /// # Formula
    ///
    /// ```ignore
    /// sum(p_j) from 0..x
    /// ```
    ///
    /// where `p_j` is the probability mass for the `j`th category
    fn cdf(&self, x: f64) -> f64 {
        assert!(x >= 0.0 && x <= self.cdf.len() as f64,
                format!("{}",
                        StatsError::ArgIntervalIncl("x", 0.0, self.cdf.len() as f64)));
        if x == self.cdf.len() as f64 {
            1.0
        } else {
            unsafe { self.cdf.get_unchecked(x as usize) / self.cdf_max() }
        }
    }
}

impl Min<u64> for Categorical {
    /// Returns the minimum value in the domain of the
    /// categorical distribution representable by a 64-bit
    /// integer
    ///
    /// # Formula
    ///
    /// ```ignore
    /// 0
    /// ```
    fn min(&self) -> u64 {
        0
    }
}

impl Max<u64> for Categorical {
    /// Returns the maximum value in the domain of the
    /// categorical distribution representable by a 64-bit
    /// integer
    ///
    /// # Formula
    ///
    /// ```ignore
    /// n
    /// ```
    fn max(&self) -> u64 {
        self.cdf.len() as u64 - 1
    }
}

impl Mean<f64> for Categorical {
    /// Returns the mean of the categorical distribution
    ///
    /// # Formula
    ///
    /// ```ignore
    /// E[X] = sum(j * p_j) for j in 0..k-1
    /// ```
    ///
    /// where `p_j` is the `j`th probability mass and `k` is the number
    /// of categoires
    fn mean(&self) -> f64 {
        self.norm_pmf.iter().enumerate().fold(0.0, |acc, (idx, &val)| acc + idx as f64 * val)
    }
}

// determines if `p` is a valid probability mass array
// for the Categorical distribution
fn is_valid_prob_mass(p: &[f64]) -> bool {
    !p.iter().any(|&x| x < 0.0 || x.is_nan()) && !p.iter().all(|&x| x == 0.0)
}

#[test]
fn test_is_valid_prob_mass() {
    let invalid = [1.0, f64::NAN, 3.0];
    assert!(!is_valid_prob_mass(&invalid));
    let invalid2 = [-2.0, 5.0, 1.0, 6.2];
    assert!(!is_valid_prob_mass(&invalid2));
    let invalid3 = [0.0, 0.0, 0.0];
    assert!(!is_valid_prob_mass(&invalid3));
    let invalid4: [f64; 0] = [];
    assert!(!is_valid_prob_mass(&invalid4));
    let valid = [5.2, 0.00001, 1e-15, 1000000.12];
    assert!(is_valid_prob_mass(&valid));
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[cfg(test)]
mod test {
    use std::fmt::Debug;
    use statistics::*;
    use distribution::{Univariate, Categorical};

    fn try_create(prob_mass: &[f64]) -> Categorical {
        let n = Categorical::new(prob_mass);
        assert!(n.is_ok());
        n.unwrap()
    }

    fn create_case(prob_mass: &[f64]) {
        try_create(prob_mass);
    }

    fn bad_create_case(prob_mass: &[f64]) {
        let n = Categorical::new(prob_mass);
        assert!(n.is_err());
    }

    fn test_case<T, F>(prob_mass: &[f64], expected: T, eval: F)
        where T: PartialEq + Debug,
              F: Fn(Categorical) -> T
    {
        let n = try_create(prob_mass);
        let x = eval(n);
        assert_eq!(expected, x);
    }

    #[test]
    fn test_create() {
        create_case(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
    }

    #[test]
    fn test_bad_create() {
        bad_create_case(&[-1.0, 1.0]);
        bad_create_case(&[0.0, 0.0]);
    }

    #[test]
    fn test_mean() {
        test_case(&[0.0, 0.25, 0.5, 0.25], 2.0, |x| x.mean());
        test_case(&[0.0, 1.0, 2.0, 1.0], 2.0, |x| x.mean());
        test_case(&[0.0, 0.5, 0.5], 1.5, |x| x.mean());
        test_case(&[0.75, 0.25], 0.25, |x| x.mean());
        test_case(&[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0], 5.0, |x| x.mean());

    }

    #[test]
    fn test_min_max() {
        test_case(&[4.0, 2.5, 2.5, 1.0], 0, |x| x.min());
        test_case(&[4.0, 2.5, 2.5, 1.0], 3, |x| x.max());
    }

    #[test]
    fn test_cdf() {
        test_case(&[0.0, 3.0, 1.0, 1.0], 3.0 / 5.0, |x| x.cdf(1.5));
        test_case(&[1.0, 1.0, 1.0, 1.0], 0.25, |x| x.cdf(0.0));
        test_case(&[4.0, 2.5, 2.5, 1.0], 0.4, |x| x.cdf(0.8));
        test_case(&[4.0, 2.5, 2.5, 1.0], 1.0, |x| x.cdf(3.2));
        test_case(&[4.0, 2.5, 2.5, 1.0], 1.0, |x| x.cdf(4.0));
    }

    #[test]
    #[should_panic]
    fn test_cdf_input_low() {
        test_case(&[4.0, 2.5, 2.5, 1.0], 1.0, |x| x.cdf(-1.0));
    }

    #[test]
    #[should_panic]
    fn test_cdf_input_high() {
        test_case(&[4.0, 2.5, 2.5, 1.0], 1.0, |x| x.cdf(4.5));
    }
}