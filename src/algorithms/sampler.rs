use rand::distributions::{Distribution, Uniform};

pub trait Sampler
{
    fn sample(&self) -> (f32, f32);
}
pub struct SimpleSampler
{
    xbounds: (f32, f32),
    ybounds: (f32, f32),

    xsampler: Uniform<f32>,
    ysampler: Uniform<f32>,

}

impl SimpleSampler
{
    pub fn new(xbounds: (f32, f32), ybounds: (f32, f32)) -> SimpleSampler
    {
        SimpleSampler {
            xbounds,
            ybounds,
            xsampler: Uniform::new(xbounds.0, xbounds.1),
            ysampler: Uniform::new(ybounds.0, ybounds.1)
        }
    }

    pub fn sample(&self) -> (f32, f32)
    {
        let mut rng = rand::thread_rng();
        (self.xsampler.sample(&mut rng), self.ysampler.sample(&mut rng))
    }
}

impl Sampler for SimpleSampler
{
    fn sample(&self) -> (f32, f32) {
        self.sample()
    }
}