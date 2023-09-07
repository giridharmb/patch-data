use tokio::time::Duration;
use tokio::time::{sleep};
use crate::yaml_data::TFEVarMetaData;
use serde_json::{json, to_string};
use my_lib;
use crate::G_MAP_WS_VARID;
use crate::G_MAP_PATCH_RESULT;
use crate::tfe::{CustomError, GenericError};
use crate::tfe::GenericError::PatchError;


const MAX_RETRIES: u32 = 5;

/*
Payload For Patch

{
    "data":
    {
        "id": "%v",
        "attributes":
        {
            "key": "%v",
            "value": "%v",
            "description": "%v",
            "category": "%v",
            "hcl": true,
            "sensitive": true
        },
        "type": "vars"
    }
}
*/


#[derive(Debug)]
pub struct TFEPatchData {
    pub key: String, // example MY_VAR_NAME
    pub value: String, // value for MY_VAR_VALUE
    pub variable_id: String, // example 'var-C39123456jybC45d'
    pub description: String, // example: 'Updated on 2023-08-28'
    pub category: String, // "env" or "terraform"
    pub sensitive: bool,
    pub hcl: bool,
}

#[derive(Debug)]
pub struct FinalPatchData {
    pub tfe_patch_data: TFEPatchData,
    pub tfe_var_metadata: TFEVarMetaData,
}

#[derive(Debug)]
pub struct TFEPatchResult {
    pub workspace_id: String,
    pub variable_id: String,
    pub host_name: String,
    pub status_code: reqwest::StatusCode,
}

fn get_json_from_tfe_patch_data(tfe_patch_data: &TFEPatchData) -> String {
    let key = &tfe_patch_data.key.to_string();
    let value = &tfe_patch_data.value.to_string();
    let variable_id = &tfe_patch_data.variable_id.to_string();
    let description = &tfe_patch_data.description.to_string();
    let category = &tfe_patch_data.category.to_string();
    let sensitive = &tfe_patch_data.sensitive;
    let hcl = &tfe_patch_data.hcl;

    let data = json!({
        "data" : {
            "id": variable_id,
            "attributes": {
                "key": key,
                "value": value,
                "description": description,
                "category": category,
                "hcl": hcl,
                "sensitive": sensitive,
            },
            "type": "vars",
        }
    });
    //data
    let json_string = to_string(&data).unwrap();
    json_string
}

pub(crate) async fn make_patch_request(final_patch_data: &FinalPatchData) -> Result<TFEPatchResult, CustomError>  {

    let url = format!("https://{}/api/v2/vars/{}", final_patch_data.tfe_var_metadata.hostname, final_patch_data.tfe_patch_data.variable_id);

    let client = reqwest::Client::new();

    // let mut var_ids_to_be_patched = vec![];

    let json_payload = get_json_from_tfe_patch_data(&final_patch_data.tfe_patch_data);

    let response = client.patch(url)
        .header("Authorization", &final_patch_data.tfe_var_metadata.token)
        .header("Content-Type", "application/vnd.api+json")
        .body(json_payload)
        .send()
        .await;

    return match response {
        Ok(d) => {
            let tfe_patch_result = TFEPatchResult {
                workspace_id: final_patch_data.tfe_var_metadata.workspace_id.to_string(),
                variable_id: final_patch_data.tfe_patch_data.variable_id.to_string(),
                host_name: final_patch_data.tfe_var_metadata.hostname.to_string(),
                status_code: d.status(),
            };
            println!("patch_result (success) >> \n\n{:#?}\n\n", d);
            Ok(tfe_patch_result)
        },
        Err(e) => {
            println!("_REQUEST_FAILED: {:#?}", e);
            let message = format!("could not patch variable_id : ({}) for workspace_id : ({}) , on host : ({})", &final_patch_data.tfe_patch_data.variable_id, &final_patch_data.tfe_var_metadata.workspace_id, &final_patch_data.tfe_var_metadata.hostname);
            let custom_err = CustomError {
                err_msg: String::from(message),
                err_type: PatchError,
            };
            println!("patch_result (failed) >> \n\n{:#?}\n\n", custom_err);
            return Err(custom_err)
        },
    };
}

pub(crate) async fn make_patch_request_with_retry(final_patch_data: FinalPatchData) -> bool {
    let mut backoff: u64 = 1;
    let mut retries = 20;

    let mut result_of_patch = false;

    loop {
        match make_patch_request(&final_patch_data).await {
            Ok(d) => {
                println!("_REQUEST_SUCCEEDED");
                G_MAP_PATCH_RESULT.insert(d.workspace_id.to_string(), d.status_code);
                result_of_patch = true;
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
    return result_of_patch
}