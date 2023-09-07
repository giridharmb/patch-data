use serde_json::{Error, from_str, Value};
use tokio::time::Duration;

use tokio::time::{sleep};
use crate::yaml_data::TFEVarMetaData;

use my_lib;
use crate::G_MAP_WS_VARID;

const MAX_RETRIES: u32 = 5;


#[derive(Debug)]
pub enum GenericError {
    JsonParseError,
    DataNotFound,
}

#[derive(Debug)]
pub struct CustomError {
    pub err_type: GenericError,
    pub err_msg: String,
}


#[derive(Debug)]
pub struct WorkspaceResult {
    pub workspace_id: String,
    pub variable_id: String,
}

pub(crate) async fn get_tfe_variable_id(tfe_data: &TFEVarMetaData) -> Result<String, CustomError> {

    let url = format!("https://{}/api/v2/workspaces/{}/vars", tfe_data.hostname, tfe_data.workspace_id);
    let client = reqwest::Client::new();


    // let mut var_ids_to_be_patched = vec![];

    let response = client.get(url)
        .header("Authorization", &tfe_data.token)
        .header("Content-Type", "application/vnd.api+json")
        .send()
        .await
        .unwrap();

    println!("{:#?}", response);

    let json_data: Result<serde_json::Value, serde_json::Error> = from_str(&response.text().await.unwrap());

    // let mut var_ids_to_be_patches = vec![];

    let mut variable_id = "";

    let result = match &json_data {
        Ok(valid_data) => {

            if let Some(my_object) = valid_data.as_object() {
                if let Some(inner_data) = my_object.get("data") {
                    if let Some(item_list) = inner_data.as_array() {

                        for item in item_list {
                            // first, get the var_id
                            let var_id = item.get("id").unwrap().as_str().unwrap();

                            // then look up item["attributes"]["key"] , and if that is equal to "ARM_CLIENT_SECRET"
                            // if it is actually "ARM_CLIENT_SECRET", then get the VAR_ID
                            if let Some(attributes) = item.get("attributes") {
                                if let Some(var_name) = attributes.get("key") {
                                    if var_name.as_str().unwrap().to_string() == tfe_data.var_name.to_string() {
                                        variable_id = var_id;
                                        println!("\n\n@workspace_id : {}\n\n{:#?}\n", tfe_data.workspace_id, item);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            let msg = format!("error in extracting json from response : {:#?}", e);
            println!("{:#?}", msg);
            let custom_err = CustomError {
                err_msg: String::from(msg),
                err_type: GenericError::JsonParseError,
            };
            return Err(custom_err);
        }
    };

    println!("{}", variable_id);

    if variable_id != "" {
        Ok(String::from(variable_id))
    } else {
        let msg = format!("could not find variable_id for {} for workspace_id : {}", tfe_data.var_name, tfe_data.workspace_id);
        println!("{:#?}", msg);
        let custom_err = CustomError {
            err_msg: String::from("could not find variable_id for workspace_id"),
            err_type: GenericError::DataNotFound,
        };
        return Err(custom_err)
    }
}

pub(crate) async fn get_tfe_variable_id_with_retry(tfe_data: TFEVarMetaData) -> String {
    let mut backoff: u64 = 1;
    let mut retries = 20;

    // let host = tfe_data.hostname;

    // let authorization_token = tfe_data.token;

    let mut result = "".to_string();
    loop {
        match get_tfe_variable_id(&tfe_data).await {
            Ok(d) => {
                println!("_REQUEST_SUCCEEDED");
                result = d.to_string();
                break;
            }
            Err(e) => {
                println!("_REQUEST_FAILED: {:#?}", e);
                retries += 1;

                if retries > MAX_RETRIES {
                    println!("_MAX_RETRIES_REACHED. EXITING...");
                    break;
                }

                println!("_RETRYING IN {} SECOND(S)...", backoff);
                sleep(Duration::from_secs(backoff)).await;

                // Exponential backoff
                backoff *= 2;
            }
        }
    }
    G_MAP_WS_VARID.insert(tfe_data.workspace_id.to_string(), result.to_string());
    return result;
}