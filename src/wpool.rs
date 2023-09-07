use futures::future::FutureExt;
use futures::pin_mut;
use futures::stream;
use futures::try_join;
use futures::{join, select, StreamExt};
use md5;
use rand::{thread_rng, Rng};
use std::fmt::{format, Formatter};
use std::time;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio;
use tokio::time::sleep as tokio_sleep;
use tokio::time::Duration as tokio_duration;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct InputData {
    pub epoch_time: u64,
    pub uuid_data: String,
    pub id: i32,
}

#[tokio::main]
async fn main() {
    let results = perform_all_calculations().await;
    println!("\nresults : \n\n{:#?}\n", results);
}

async fn perform_all_calculations() -> Vec<String> {
    let now = time::Instant::now();
    let input_vec = generate_input_data();
    let input_vec_length = input_vec.len();
    // at any given time, run these many async functions
    let concurrency = 8;
    let results: Vec<_> = stream::iter(input_vec)
        .map(perform_calculation_for_input)
        .buffer_unordered(concurrency)
        .collect()
        .await;
    let elapsed = now.elapsed().as_secs_f64();
    println!(
        "perform_all_calculations() : total time taken : {}",
        elapsed
    );
    results
}

// generate a vector of input data for computation
fn generate_input_data() -> Vec<InputData> {
    let mut input_data = vec![];
    for i in 1..=16 {
        let current_epoch = get_epoch_time();
        let random_uuid = Uuid::new_v4();
        let input = InputData {
            epoch_time: current_epoch,
            uuid_data: random_uuid.to_string(),
            id: i,
        };
        input_data.push(input);
    }
    println!("generate_input_data() : input_data : \n{:#?}\n", input_data);
    input_data
}

// for a given input of type (InputData), compute md5 hash
async fn perform_calculation_for_input(input: InputData) -> String {
    // first sleep for some random seconds (to simulate doing some job)
    let random_number = get_random_number();

    tokio_sleep(tokio_duration::from_secs_f64(random_number)).await;

    // if epoch is 1234 and uuid is a0b1c2d3 , then input_str will be 1234_a0b1c2d3
    let input_str = input.to_string();
    // let input_str = get_input_data_as_string(input.clone());

    // this will compute md5 of 1234_a0b1c2d3
    let computed_md5 = get_md5(input_str);

    // print the output
    println!(
        "perform_calculation_for_input() : input : {:#?} , \ncomputed_md5 : {}\n\n",
        input, computed_md5
    );

    // return the computed md5 hash as a string
    computed_md5
}

// helper function : get current epoch time
fn get_epoch_time() -> u64 {
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();
    since_the_epoch.as_secs()
}

// helper function : get md5 hash of a given input string
fn get_md5(my_str: String) -> String {
    let digest = md5::compute(my_str);
    let computed_hash = format!("{:x}", digest);
    computed_hash
}

// helper function : get InputData as a string
impl std::fmt::Display for InputData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}", self.epoch_time, self.uuid_data)
    }
}

// helper function : get random number
fn get_random_number() -> f64 {
    let mut rng = thread_rng();
    let random_number = rng.gen_range(2..5);
    random_number as f64
}
