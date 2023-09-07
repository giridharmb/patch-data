mod fdp;
mod tfe;
mod wpool;
mod yaml_data;
mod tfe_v2;
mod tfe_patch;

use std::collections::HashMap;
use reqwest;
use futures::future::FutureExt;
use futures::pin_mut;
use futures::stream;
use futures::try_join;
use futures::{join, select, StreamExt};
use md5;
use rand::{thread_rng, Rng};
use std::fmt::{format, Formatter};
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::time;
use std::time::{SystemTime, UNIX_EPOCH};
use lazy_static::lazy_static;
use tokio;
use tokio::sync::RwLock;
use tokio::time::sleep as tokio_sleep;
use tokio::time::Duration as tokio_duration;
use uuid::Uuid;
use crate::tfe::{CustomError};

use my_lib;
use crate::tfe_v2::get_tfe_variable_id_with_retry;
use crate::yaml_data::{get_workspace_data, TFEVarMetaData};
use dashmap;
use serde_json::to_string_pretty;
use reqwest::StatusCode;
use crate::tfe_patch::{FinalPatchData, make_patch_request_with_retry, TFEPatchData};


lazy_static! {
    pub static ref G_MAP_WS_VARID: dashmap::DashMap<String, String> = dashmap::DashMap::new();
    pub static ref G_MAP_PATCH_RESULT: dashmap::DashMap<String, reqwest::StatusCode> = dashmap::DashMap::new();
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct InputData {
    pub epoch_time: u64,
    pub uuid_data: String,
    pub id: i32,
}

#[tokio::main]
async fn main() {

    let mut my_map: HashMap<String, String> = HashMap::new();

    let results = get_tfe_variable_ids_in_parallel().await;

    /* ************************************************************************* */

    for entry in G_MAP_WS_VARID.iter() {
        println!("{} : {}", entry.key(), entry.value());
        my_map.insert(entry.key().to_string(), entry.value().to_string());
    }

    // Serialize the HashMap to a JSON string
    let json_str = to_string_pretty(&my_map).expect("Failed to serialize");

    // Save the JSON string to a file
    let mut file = File::create("map.json").expect("Failed to create file");
    file.write_all(json_str.as_bytes()).expect("Failed to write to file");

    println!("HashMap has been saved to map.json");

    /* ************************************************************************* */

    get_final_patch_data_list();

    let patch_results = patch_all_variables_in_parallel().await;

    println!("{:#?}",patch_results);

}



pub fn get_final_patch_data_list() -> Vec<FinalPatchData> {
    let mut final_list: Vec<FinalPatchData> = vec![];

    let input_vec = yaml_data::get_workspace_data();

    let mut map_ws_id_var_id: HashMap<String, String> = HashMap::new();

    for entry in G_MAP_WS_VARID.iter() {
        println!("{} : {}", entry.key(), entry.value());
        map_ws_id_var_id.insert(entry.key().to_string(), entry.value().to_string());
    }

    for workspace_yaml_data in input_vec {

        let ws_id_from_yaml = workspace_yaml_data.workspace_id.to_string();
        let var_id_for_ws_id = map_ws_id_var_id.get(ws_id_from_yaml.as_str()).unwrap();


        let tfe_var_metadata = TFEVarMetaData {
            workspace_id: ws_id_from_yaml.to_string(),
            hostname: workspace_yaml_data.hostname.to_string(),
            token: workspace_yaml_data.token.to_string(),
            var_name: workspace_yaml_data.var_name.to_string(),
            var_value: workspace_yaml_data.var_value.to_string(),
        };

        let tfe_patch_data = TFEPatchData {
            key: workspace_yaml_data.var_name.to_string(),
            value: workspace_yaml_data.var_value.to_string(),
            variable_id: var_id_for_ws_id.to_string(),
            description: "patched_on_2023_09_06".to_string(),
            category: "env".to_string(),
            sensitive: true,
            hcl: false,
        };

        let final_patch_data = FinalPatchData {
            tfe_patch_data,
            tfe_var_metadata,
        };

        final_list.push(final_patch_data);
    }

    // println!("{:#?}",final_list);

    final_list
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

async fn get_tfe_variable_ids_in_parallel() -> Vec<String> {
    let now = time::Instant::now();
    let input_vec = yaml_data::get_workspace_data();
    let input_vec_length = input_vec.len();
    // at any given time, run these many async functions
    let concurrency = 8;
    let results: Vec<_> = stream::iter(input_vec)
        .map(get_tfe_variable_id_with_retry)
        .buffer_unordered(concurrency)
        .collect()
        .await;
    let elapsed = now.elapsed().as_secs_f64();
    println!(
        "exec_v2() : total time taken : {}",
        elapsed
    );
    results
}

async fn patch_all_variables_in_parallel() -> Vec<bool> {
    let now = time::Instant::now();
    let input_vec = get_final_patch_data_list();
    let input_vec_length = input_vec.len();
    // at any given time, run these many async functions
    let concurrency = 8;
    let results: Vec<_> = stream::iter(input_vec)
        .map(make_patch_request_with_retry)
        .buffer_unordered(concurrency)
        .collect()
        .await;
    let elapsed = now.elapsed().as_secs_f64();
    println!(
        "patch_all_variables_in_parallel() : total time taken : {}",
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
