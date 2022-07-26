use rand::{SeedableRng, Rng};
use rand_hc::Hc128Rng;

pub fn test() {

    let mut rng = Hc128Rng::from_entropy();
    let n = rng.gen_range(0..10);
    dbg!(n);

    //let rng = Hc128Rng::from_seed(arr);


    // let rng = Hc128Rng::from_rng(&mut rand::thread_rng());   
    // let mut res = rng.unwrap();
    // dbg!(res.gen_range(0..10));
    
    
}