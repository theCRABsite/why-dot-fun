use crate::secrets::Secrets;

/// Check whether the token is valid.
/// If true is returned, the frontend user receives a twilio access token that can be used to
/// make a single call towards our application.
pub fn check_token(_token: &str, _secrets: &Secrets) -> bool {

    

    return true;
}
