#### Generic Terraform Variable Patch Tool

- This tool fetches Variable IDs For Given List of Workspaces In Defined In `work_spaces.yaml`
- Then It Issues an HTTP Patch Request For Those VariableIDs With Key And Value Defined In `work_spaces.yaml`
- Has Exponential Re-Try Logic (For HTTP Request)
- Concurrency On Async Functions Using `futures::stream`
- There Is A Lot of Extra Code, Please Ignore That

Before Running This Tool, Make Sure You Have The File `work_spaces.yaml` Populated With Relevant Data.

```bash
workspace_id : Terraform Workspace ID
hostname     : Terraform Host Name
token        : Terraform Admin Token
var_name     : Terraform Workspace Variable Name
var_value    : Terraform Workspace Variable Value (the current value will be replaced with this value)
```

Built The Binary

```bash
cargo build --release
```

File : `work_spaces.yaml` (This is what is required)

```yaml
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
```

File : `workspace_ids.yaml` (This is not used, You don't have to create this file)

```yaml
workspace_ids:
  - ws-xyzabcd1000
  - ws-xyzabcd1001
  - ws-xyzabcd1002
  - ws-xyzabcd1003
  - ws-xyzabcd1004
  - ws-xyzabcd1005
```

Run The Patch Tool

```bash
target/release/patch-data
```

Example Of Concurrency On Async Functions

```rust
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
```
