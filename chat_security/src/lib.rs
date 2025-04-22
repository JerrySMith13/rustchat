use rand::{rngs::OsRng, CryptoRng};
use rand::RngCore;
use x25519_dalek::{EphemeralSecret, PublicKey};


struct SessionCrypt{
    pk: PublicKey,
    sk: EphemeralSecret,
}

impl SessionCrypt{
    pub fn new<R>(mut rng: R) -> Self
    where R: RngCore + CryptoRng
    {
        let sk = EphemeralSecret::random_from_rng(&mut rng);
        let pk = PublicKey::from(&sk);
        SessionCrypt {
            pk,
            sk,
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
