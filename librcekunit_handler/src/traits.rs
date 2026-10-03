use crate::enums::HttpMethod;
use crate::error::Result;
use crate::types::{InputUserId, PicId, UserId};
use core::future::Future;
use std::collections::HashMap;

pub type Form = HashMap<String, String>;

pub trait Transport {
    type Response;

    fn request(
        &self,
        method: HttpMethod,
        path: &str,
        body: Option<Form>,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    fn fetch_csrf_token(&self, url: &str) -> impl Future<Output = Result<String>> + Send;

    fn reset_csrf_token(&self) -> impl Future<Output = ()> + Send;

    fn clear_session(&self) -> impl Future<Output = ()> + Send;
}

pub trait Auth {
    fn login(&self, email: &str, password: &str) -> impl Future<Output = Result<()>> + Send;
    fn logout(&self) -> impl Future<Output = Result<()>> + Send;
}

pub trait Crud {
    type Response;

    fn index(&self) -> impl Future<Output = Result<Self::Response>> + Send;

    fn index_with_params(
        &self,
        params: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    fn create(&self) -> impl Future<Output = Result<Self::Response>> + Send;

    fn store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;

    fn show(&self, id: u64) -> impl Future<Output = Result<Self::Response>> + Send;

    fn edit(&self, id: u64) -> impl Future<Output = Result<Self::Response>> + Send;

    fn update(&self, id: u64, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;

    fn destroy(&self, id: u64) -> impl Future<Output = Result<Self::Response>> + Send;
}

pub trait Dashboard {
    type Response;

    fn dashboard_index(&self) -> impl Future<Output = Result<Self::Response>> + Send;

    fn dashboard_index_with_params(
        &self,
        params: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    fn delete_all(&self) -> impl Future<Output = Result<Self::Response>> + Send;

    fn delete_by_category(
        &self,
        column: &str,
        value: &str,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    fn export(
        &self,
        format: &str,
        sort: &str,
        direction: &str,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    fn get_unique_values(
        &self,
        column: &str,
    ) -> impl Future<Output = Result<Self::Response>> + Send;
}

pub trait InputData {
    type Response;

    fn input_data_create(&self) -> impl Future<Output = Result<Self::Response>> + Send;

    fn input_data_store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;
}

pub trait InputUser {
    type Response;

    fn input_user_index(&self) -> impl Future<Output = Result<Self::Response>> + Send;

    fn input_user_index_with_params(
        &self,
        params: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    fn input_user_create(&self) -> impl Future<Output = Result<Self::Response>> + Send;

    fn input_user_store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;

    fn input_user_show(
        &self,
        id: InputUserId,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    fn input_user_edit(
        &self,
        id: InputUserId,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    fn input_user_update(
        &self,
        id: InputUserId,
        data: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    fn input_user_destroy(
        &self,
        id: InputUserId,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    fn input_user_export(
        &self,
        params: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    fn input_user_import(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;
}

pub trait Pic {
    type Response;

    fn pic_index(&self) -> impl Future<Output = Result<Self::Response>> + Send;

    fn pic_index_with_params(
        &self,
        params: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    fn pic_create(&self) -> impl Future<Output = Result<Self::Response>> + Send;

    fn pic_store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;

    fn pic_show(&self, id: PicId) -> impl Future<Output = Result<Self::Response>> + Send;

    fn pic_edit(&self, id: PicId) -> impl Future<Output = Result<Self::Response>> + Send;

    fn pic_update(
        &self,
        id: PicId,
        data: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    fn pic_destroy(&self, id: PicId) -> impl Future<Output = Result<Self::Response>> + Send;

    fn dashboard_pic_index(&self) -> impl Future<Output = Result<Self::Response>> + Send;

    fn dashboard_pic_index_with_params(
        &self,
        params: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    fn input_pic_create(&self) -> impl Future<Output = Result<Self::Response>> + Send;

    fn input_pic_store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;
}

pub trait Users {
    type Response;

    fn users_index(&self) -> impl Future<Output = Result<Self::Response>> + Send;

    fn users_index_with_params(
        &self,
        params: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    fn users_create(&self) -> impl Future<Output = Result<Self::Response>> + Send;

    fn users_store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send;

    fn users_show(&self, id: UserId) -> impl Future<Output = Result<Self::Response>> + Send;

    fn users_edit(&self, id: UserId) -> impl Future<Output = Result<Self::Response>> + Send;

    fn users_update(
        &self,
        id: UserId,
        data: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    fn users_destroy(&self, id: UserId) -> impl Future<Output = Result<Self::Response>> + Send;

    fn dashboard_users_index(&self) -> impl Future<Output = Result<Self::Response>> + Send;

    fn dashboard_users_index_with_params(
        &self,
        params: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send;
}
