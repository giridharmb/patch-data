#[macro_use]

use std::fs::File;
use std::io::prelude::*;
use serde_derive::Deserialize;
use serde::Deserialize;
use crate::tfe_patch::FinalPatchData;

#[derive(Debug, Deserialize)]
struct Config {
    workspace_ids: Vec<String>,
}

pub(crate) fn get_workspace_ids() -> Vec<String> {

    let mut workspace_ids = vec![];

    /*
        workspace_ids.yaml:

          workspace_ids:
           - ws-xyzabcd1000
           - ws-xyzabcd1001
           - ws-xyzabcd1002
           - ws-xyzabcd1003
           - ws-xyzabcd1004
           - ws-xyzabcd1005
     */

    // Open the YAML file
    let mut file = File::open("workspace_ids.yaml").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    // Deserialize the YAML content into a Rust structure
    let config: Config = serde_yaml::from_str(&contents).unwrap();

    // Print the hostnames
    for hostname in &config.workspace_ids {
        // println!("{}", hostname);
        workspace_ids.push(hostname.to_string())
    }
    workspace_ids
}


#[derive(Debug, Deserialize)]
pub struct TFEVarMetaData {
    pub workspace_id: String,
    pub hostname: String,
    pub token: String,
    pub var_name: String,
    pub var_value: String,
}

pub fn get_workspace_data() -> Vec<TFEVarMetaData> {
    /*
        work_spaces.yaml:

            ---
              - workspace_id: "ws-xyzabcd0000001"
                hostname: "my-host-001.company.com"
                token: "Bearer <MY_TOKEN>"
                var_name: "MY_VAR_NAME"
                var_value: "MY_VALUE_FOR_VAR"
              - workspace_id: "ws-xyzabcd0000002"
                hostname: "my-host-001.company.com"
                token: "Bearer <MY_TOKEN>"
                var_name: "MY_VAR_NAME"
                var_value: "MY_VALUE_FOR_VAR"
              - workspace_id: "ws-xyzabcd0000003"
                hostname: "my-host-001.company.com"
                token: "Bearer <MY_TOKEN>"
                var_name: "MY_VAR_NAME"
                var_value: "MY_VALUE_FOR_VAR"
    */

    // Open the YAML file
    let mut file = File::open("work_spaces.yaml").unwrap();
    let mut contents = String::new();

    // Read the file content
    file.read_to_string(&mut contents).unwrap();

    // Deserialize YAML into a Vec<User>
    let tfe_data_items: Vec<TFEVarMetaData> = serde_yaml::from_str(&contents).unwrap();

    tfe_data_items
}