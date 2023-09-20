use super::NUMERIC_LEN;
use fss::ic::*;
use fss::beavertuple::BeaverTuple;
use fss::{Group, RingElm};
use super::{write_file, read_file};
use fss::{bits_to_u32, bits_to_u32_BE};
use fss::prg::FixedKeyPrgStream;
//use std::path::PathBuf;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaxOffline_IC{
    pub alpha : Vec<RingElm>,
    pub ic_key: Vec<ICCKey>,
    pub beavers: Vec<BeaverTuple>
} 

impl MaxOffline_IC{
    pub fn new() -> Self{
        Self{alpha: Vec::<RingElm>::new(), ic_key: Vec::<ICCKey>::new(), beavers: Vec::<BeaverTuple>::new()}
    }

    pub fn genData(stream: &mut FixedKeyPrgStream, ic_key_size: usize, cbeavers_num: usize){
        //let alpha_bits = stream.next_bits(32usize);
        let (p_bound,q_bound) = (RingElm::zero(), RingElm::from((1<<31)-1));
        let mut alpha0 = Vec::<RingElm>::new();
        let mut alpha1 = Vec::<RingElm>::new();
        let mut ic_key_0 = Vec::<ICKey>::new();
        let mut ic_key_1 = Vec::<ICKey>::new();
        for _ in 0..ic_key_size{
            let alpha_bits = stream.next_bits(NUMERIC_LEN);
            let (key0, key1) = ICKey::gen(&alpha_bits,&p_bound, &q_bound);
            ic_key_0.push(key0);
            ic_key_1.push(key1);
            let alpha_bits_share = stream.next_bits(NUMERIC_LEN);
            let mut alpha_numeric = RingElm::from(bits_to_u32_BE(&alpha_bits));
            let mut alpha_share = RingElm::from(bits_to_u32_BE(&alpha_bits_share));
            alpha_numeric.sub(&alpha_share);
            alpha0.push(alpha_share);
            alpha1.push(alpha_numeric);
        }
        
        write_file("../data/ic_key0.bin", &ic_key_0);
        write_file("../data/ic_key1.bin", &ic_key_1);
        write_file("../data/alpha0.bin", &alpha0);
        write_file("../data/alpha1.bin", &alpha1);

        let mut beavertuples0: Vec<BeaverTuple> = Vec::new();
        let mut beavertuples1: Vec<BeaverTuple> = Vec::new();
        for i in 0..cbeavers_num{
            let rd_bits = stream.next_bits(NUMERIC_LEN*5);
            let a0 = RingElm::from( bits_to_u32(&rd_bits[..NUMERIC_LEN]) );
            let b0 = RingElm::from( bits_to_u32(&rd_bits[NUMERIC_LEN..2*NUMERIC_LEN]) );

            let a1 = RingElm::from( bits_to_u32(&rd_bits[2*NUMERIC_LEN..3*NUMERIC_LEN]) );
            let b1 = RingElm::from( bits_to_u32(&rd_bits[3*NUMERIC_LEN..4*NUMERIC_LEN]));

            let ab0 = RingElm::from( bits_to_u32(&rd_bits[4*NUMERIC_LEN..5*NUMERIC_LEN]) );

            let mut a = RingElm::zero();
            a.add(&a0);
            a.add(&a1);

            let mut b = RingElm::zero();
            b.add(&b0);
            b.add(&b1);

            let mut ab = RingElm::one();
            ab.mul(&a);
            ab.mul(&b);

            ab.sub(&ab0);

            let beaver0 = BeaverTuple{
                a: a0,
                b: b0,
                ab: ab0,
                delta_a:RingElm::zero(),
                delta_b:RingElm::zero(),
            };

            let beaver1 = BeaverTuple{
                a: a1,
                b: b1,
                ab: ab,
                delta_a:RingElm::zero(),
                delta_b:RingElm::zero(),
            };
            beavertuples0.push(beaver0);
            beavertuples1.push(beaver1);
        }
        write_file("../data/beaver0.bin", &beavertuples0);
        write_file("../data/beaver1.bin", &beavertuples1);
    }

    pub fn loadData(&mut self,idx:&u8){
        match read_file(&format!("../data/alpha{}.bin", idx)) {
            Ok(value) => self.alpha = value,
            Err(e) => println!("Error reading file: {}", e),  // Or handle the error as needed
        }
        
        match read_file(&format!("../data/ic_key{}.bin", idx)) {
            Ok(value) => self.ic_key = value,
            Err(e) => println!("Error reading file: {}", e),  // Or handle the error as needed
        }

        match read_file(&format!("../data/beaver{}.bin", idx)) {
            Ok(value) => self.beavers = value,
            Err(e) => println!("Error reading file: {}", e),  // Or handle the error as needed
        }
    }
}

#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn gen_data(){
        
    }
}