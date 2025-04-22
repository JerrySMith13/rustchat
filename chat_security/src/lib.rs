use rand::{rngs::StdRng, SeedableRng};
use x25519_dalek::{EphemeralSecret, PublicKey};


fn generate_keys() {
    let rng = StdRng::from_os_rng();
    let secret = EphemeralSecret::random_from_rng(rng);
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
