use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub phone: String,
    pub avatar: String,
    pub location: String,
    pub age: i32,
    pub gender: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginModel {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterModel {
    pub username: String,
    pub password: String,
    pub confirm_password: String,
    pub email: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoUpdateModel {
    pub username: String,
    pub email: String,
    pub phone: String,
    pub avatar: String,
    pub location: String,
    pub age: i32,
    pub gender: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordUpdateModel {
    pub original_password: String,
    pub new_password: String,
    pub confirm_password: String,
}
