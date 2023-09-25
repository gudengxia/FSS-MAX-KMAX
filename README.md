This project is a fork from https://github.com/zevenblade/FSS-MAX, and contributes to securely compute the max value and the k-th max value of an array that is shared between two parties.
 
This project assumes k_share is an arithmetical share between the two parties. And in the test mod in frontend/src/main, run gen_data_for_max_ic() to generate the offline data. Then, run heap_sort to sort the x_share in an incremental order, then run kmax_ic() to get the k-th max value. 

# MPC
This project is to perform secure computation for the maximum/k^{th} ranking element from a secret set.

The are a few directories in the solution:
  - data: store the generated offline data
  - test: store auto-generated binary files for test purpose
  - libfss: implmentation for those fss primitives 
  - offline: assisting in generating offline data used for online 2PC
  - frontend: A frontend for bencharmking all basic protocols in libmpc.

To generate offline data (for batch_max), Open one terminal:
    - cd frontend
    - cargo test batch_max_gen_offlinedata

Open two terminals
    - cd frontend
    - In the 1st terminal: cargo run 0
    - In the 2nd terminal: cargo run 1

The results are written in "test", in frontend repo then run "cargo test test_result" to verify the results.

