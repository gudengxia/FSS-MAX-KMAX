use std::{f64::consts::LOG2_10, rc};

use crate::offline_data::offline_ic_max::MaxOffline_IC;
use super::super::mpc_party::*;
use fss::{RingElm, beavertuple::BeaverTuple};

pub async fn max_ic(p: &mut MPCParty<MaxOffline_IC>, x: &Vec<RingElm>) ->RingElm{
    let mut x_share = x.clone();
    let x_len = x_share.len();
    let mut t = Vec::<RingElm>::new();
    
    let is_server = p.netlayer.is_server;
    let mut ic_key_it = p.offlinedata.ic_key.iter();
    let mut alpha_it = p.offlinedata.alpha.iter();
    let mut beaver_it = p.offlinedata.beavers.iter();
    
    while x_len > 1{
        t.clear();
        /**************************************START COMPUTE LESSTHAN****************************/
        let mut msg_share_x_ic = Vec::<RingElm>::new(); // to store the masked value that is  x[i*2]-x[i*2+1]+alpha
        for i in 0..x_len/2{
            let x_diff = x_share[i*2] - x_share[i*2+1];
            let alpha = alpha_it.next().expect("No enough alpha to use.");
            let x_ic = x_diff +  alpha.clone();
            msg_share_x_ic.push(x_ic);
        } // prepare the message to exchange
        
        let x_ics = p.netlayer.exchange_ring_vec(msg_share_x_ic).await; // exchange (x[i*2]-x[i*2+1]) to get n/2 points for LessThan function
        
        //compute n/2 x-b*(x-y), where b is the eval result of the LessThen function. It needs to exchange beaver tuples
        let mut my_beavers = Vec::<BeaverTuple>::new(); // to store n/2 beaver tuples for multiplications
        let mut msg_share_beaver = Vec::<RingElm>::new(); 
        for i in 0..x_len/2{
            let x_diff = x_share[i*2] - x_share[i*2+1];
            let ic_key = ic_key_it.next().expect("No enough ic_key.");
            let y_ic = ic_key.eval(&x_ics[i]);
            let mut beaver = beaver_it.next().expect("No enough beaver tuple.").clone();
            let half_beaver = beaver.mul_open(y_ic, x_diff);
            msg_share_beaver.push(half_beaver.0);
            msg_share_beaver.push(half_beaver.1);
            my_beavers.push(beaver);
        } 
        
        let msg_beavers = p.netlayer.exchange_ring_vec(msg_share_beaver).await;

        for i in 0..x_len/2{
            let mul_result = my_beavers[i].mul_compute(is_server, &msg_beavers[i*2], &msg_beavers[i*2+1]);
            let max_of_two = x_share[i*2] - mul_result;
            t.push(max_of_two); 
        }
        /**************************************END   COMPUTE LESSTHAN****************************/    
        
        // deal with the last element if x_len is odd
        if x_len & 0x1usize == 1{
            t.push(x_share[x_len-1]);
        } 

        //update x_share
        x_share.clear();
        x_share.extend(t.clone());
    }
    x_share[0]  
}

/****************************************************************************************************************************************************/
 /**We regard the array as a logic tree, where the i-th element's parent is the (i/2)-th element. By comparing every element to its parent,**********/ 
 /**We let the max value be the parent and the min value is the child.                                                                              */
 /****************************************************************************************************************************************************/
pub async fn heapify(p: &mut MPCParty<MaxOffline_IC>, x_share: &mut Vec<RingElm>) -> RingElm{
    let x_len = x_share.len();
    
    let is_server = p.netlayer.is_server;
    let mut ic_key_it = p.offlinedata.ic_key.iter();
    let mut alpha_it = p.offlinedata.alpha.iter();
    let mut beaver_it = p.offlinedata.beavers.iter();
    
    let h = ((x_len+1) as f64).log(2 as f64).ceil() as usize; //the depth of the logic tree that has x_len nodes   
    let mut start_index = 2 << (h-1) - 1; //the start index of the nodes to be handled
    let mut end_index = x_len - 1;  //the end index of the nodes to be handled
    for i in (1..h).rev(){
        let rchildren = ((start_index+1)..=end_index).step_by(2);
        let lchildren = ((start_index)..=end_index).step_by(2);

        /*******************************START: COMPARE THE RIGHT CHILDREN TO THEIR PARENT****************************************/
        let handle_rchildren = {
            let mut msg_share_x_ic = Vec::<RingElm>::new(); // to store the masked value that is  x[parent]-x[j]+alpha
            for j in rchildren.clone(){
                let parent = j / 2;
                let x_diff = x_share[parent] - x_share[j];
                let alpha = alpha_it.next().expect("No enough alpha to use.");
                let x_ic = x_diff +  alpha.clone();
                msg_share_x_ic.push(x_ic);
            }

            let x_ics = p.netlayer.exchange_ring_vec(msg_share_x_ic).await;

            let mut my_beavers = Vec::<BeaverTuple>::new(); // to store n/2 beaver tuples for multiplications
            let mut msg_share_beaver = Vec::<RingElm>::new();
        
            let mut x_ics_it = x_ics.iter();  
            for j in  rchildren.clone(){
                let parent = j / 2;
                let x_diff = x_share[parent] - x_share[j];
                let ic_key = ic_key_it.next().expect("No enough ic_key.");
                let x_ic = x_ics_it.next().expect("No enough x_ic.");
                let y_ic = ic_key.eval(x_ic);
                let mut beaver = beaver_it.next().expect("No enough beaver tuple.").clone();
                let half_beaver = beaver.mul_open(y_ic, x_diff);
                msg_share_beaver.push(half_beaver.0);
                msg_share_beaver.push(half_beaver.1);
                my_beavers.push(beaver);
            }

            let msg_beavers = p.netlayer.exchange_ring_vec(msg_share_beaver).await;
            let mut mul_index= 0;
            for j in rchildren.clone(){
                let parent = j / 2;
                let mul_result = my_beavers[mul_index].mul_compute(is_server, &msg_beavers[mul_index*2] ,&msg_beavers[mul_index*2+1]);
                mul_index += 1;
                let max_of_two = x_share[parent] - mul_result;
                let sum_of_two = x_share[parent] + x_share[j];
                x_share[parent] = max_of_two;
                x_share[j] = sum_of_two - max_of_two;
            }
        };
        /*******************************END: COMPARE THE RIGHT CHILDREN TO THEIR PARENT******************************************/

        /*******************************START: COMPARE THE LEFT CHILDREN TO THEIR PARENT*******************************************/
        let handle_lchildren = {
            let mut msg_share_x_ic = Vec::<RingElm>::new(); // to store the masked value that is  x[parent]-x[j]+alpha
            for j in rchildren.clone(){
                let parent = j / 2;
                let x_diff = x_share[parent] - x_share[j];
                let alpha = alpha_it.next().expect("No enough alpha to use.");
                let x_ic = x_diff +  alpha.clone();
                msg_share_x_ic.push(x_ic);
            }

            let x_ics = p.netlayer.exchange_ring_vec(msg_share_x_ic).await;

            let mut my_beavers = Vec::<BeaverTuple>::new(); // to store n/2 beaver tuples for multiplications
            let mut msg_share_beaver = Vec::<RingElm>::new();
        
            let mut x_ics_it = x_ics.iter();  
            for j in  rchildren.clone(){
                let parent = j / 2;
                let x_diff = x_share[parent] - x_share[j];
                let ic_key = ic_key_it.next().expect("No enough ic_key.");
                let x_ic = x_ics_it.next().expect("No enough x_ic.");
                let y_ic = ic_key.eval(x_ic);
                let mut beaver = beaver_it.next().expect("No enough beaver tuple.").clone();
                let half_beaver = beaver.mul_open(y_ic, x_diff);
                msg_share_beaver.push(half_beaver.0);
                msg_share_beaver.push(half_beaver.1);
                my_beavers.push(beaver);
            }

            let msg_beavers = p.netlayer.exchange_ring_vec(msg_share_beaver).await;
            let mut mul_index= 0;
            for j in rchildren.clone(){
                let parent = j / 2;
                let mul_result = my_beavers[mul_index].mul_compute(is_server, &msg_beavers[mul_index*2] ,&msg_beavers[mul_index*2+1]);
                mul_index += 1;
                let max_of_two = x_share[parent] - mul_result;
                let sum_of_two = x_share[parent] + x_share[j];
                x_share[parent] = max_of_two;
                x_share[j] = sum_of_two - max_of_two;
            }
        };
        /*******************************END: COMPARE THE LEFT CHILDREN TO THEIR PARENT*********************************************/
        end_index = start_index - 1;
        start_index = 2 << i - 1;
    }
    return x_share[0];
}