const MAX_RETRIES: u32 = 5;


#[derive(Debug)]
pub enum GenericError {
    JsonParseError,
    DataNotFound,
    PatchError,
}

#[derive(Debug)]
pub struct CustomError {
    pub err_type: GenericError,
    pub err_msg: String,
}
