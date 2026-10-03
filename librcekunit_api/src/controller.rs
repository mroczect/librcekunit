use core::future::Future;

use librcekunit_handler::{
    Auth, Crud, Dashboard, Form, InputData, InputUser, InputUserId, Pic, PicId, Result, UserId,
    Users,
};

use crate::client::Client;
use crate::endpoints;

impl Auth for Client {
    fn login(&self, email: &str, password: &str) -> impl Future<Output = Result<()>> + Send {
        endpoints::auth::login(self.http(), email, password)
    }

    fn logout(&self) -> impl Future<Output = Result<()>> + Send {
        endpoints::auth::logout(self.http())
    }
}

impl Crud for Client {
    type Response = reqwest::Response;

    fn index(&self) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::cekunit::index(self.http())
    }

    fn index_with_params(
        &self,
        params: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::cekunit::index_with_params(self.http(), params)
    }

    fn create(&self) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::cekunit::create(self.http())
    }

    fn store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::cekunit::store(self.http(), data)
    }

    fn show(&self, id: u64) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::cekunit::show(self.http(), id)
    }

    fn edit(&self, id: u64) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::cekunit::edit(self.http(), id)
    }

    fn update(&self, id: u64, data: Form) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::cekunit::update(self.http(), id, data)
    }

    fn destroy(&self, id: u64) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::cekunit::destroy(self.http(), id)
    }
}

impl Dashboard for Client {
    type Response = reqwest::Response;

    fn dashboard_index(&self) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::dashboard::index(self.http())
    }

    fn dashboard_index_with_params(
        &self,
        params: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::dashboard::index_with_params(self.http(), params)
    }

    fn delete_all(&self) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::dashboard::delete_all(self.http())
    }

    fn delete_by_category(
        &self,
        column: &str,
        value: &str,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::dashboard::delete_by_category(self.http(), column, value)
    }

    fn export(
        &self,
        format: &str,
        sort: &str,
        direction: &str,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::dashboard::export(self.http(), format, sort, direction)
    }

    fn get_unique_values(
        &self,
        column: &str,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::dashboard::get_unique_values(self.http(), column)
    }
}

impl InputData for Client {
    type Response = reqwest::Response;

    fn input_data_create(&self) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::input_data::create(self.http())
    }

    fn input_data_store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::input_data::store(self.http(), data)
    }
}

impl InputUser for Client {
    type Response = reqwest::Response;

    fn input_user_index(&self) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::input_user::index(self.http())
    }

    fn input_user_index_with_params(
        &self,
        params: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::input_user::index_with_params(self.http(), params)
    }

    fn input_user_create(&self) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::input_user::create(self.http())
    }

    fn input_user_store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::input_user::store(self.http(), data)
    }

    fn input_user_show(
        &self,
        id: InputUserId,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::input_user::show(self.http(), id.get())
    }

    fn input_user_edit(
        &self,
        id: InputUserId,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::input_user::edit(self.http(), id.get())
    }

    fn input_user_update(
        &self,
        id: InputUserId,
        data: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::input_user::update(self.http(), id.get(), data)
    }

    fn input_user_destroy(
        &self,
        id: InputUserId,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::input_user::destroy(self.http(), id.get())
    }

    fn input_user_export(
        &self,
        params: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::input_user::export(self.http(), params)
    }

    fn input_user_import(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::input_user::import(self.http(), data)
    }
}

impl Pic for Client {
    type Response = reqwest::Response;

    fn pic_index(&self) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::pic::index(self.http())
    }

    fn pic_index_with_params(
        &self,
        params: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::pic::index_with_params(self.http(), params)
    }

    fn pic_create(&self) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::pic::create(self.http())
    }

    fn pic_store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::pic::store(self.http(), data)
    }

    fn pic_show(&self, id: PicId) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::pic::show(self.http(), id.get())
    }

    fn pic_edit(&self, id: PicId) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::pic::edit(self.http(), id.get())
    }

    fn pic_update(
        &self,
        id: PicId,
        data: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::pic::update(self.http(), id.get(), data)
    }

    fn pic_destroy(&self, id: PicId) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::pic::destroy(self.http(), id.get())
    }

    fn dashboard_pic_index(&self) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::pic::dashboard_index(self.http())
    }

    fn dashboard_pic_index_with_params(
        &self,
        params: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::pic::dashboard_index_with_params(self.http(), params)
    }

    fn input_pic_create(&self) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::pic::input_create(self.http())
    }

    fn input_pic_store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::pic::input_store(self.http(), data)
    }
}

impl Users for Client {
    type Response = reqwest::Response;

    fn users_index(&self) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::users::index(self.http())
    }

    fn users_index_with_params(
        &self,
        params: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::users::index_with_params(self.http(), params)
    }

    fn users_create(&self) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::users::create(self.http())
    }

    fn users_store(&self, data: Form) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::users::store(self.http(), data)
    }

    fn users_show(&self, id: UserId) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::users::show(self.http(), id.get())
    }

    fn users_edit(&self, id: UserId) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::users::edit(self.http(), id.get())
    }

    fn users_update(
        &self,
        id: UserId,
        data: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::users::update(self.http(), id.get(), data)
    }

    fn users_destroy(&self, id: UserId) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::users::destroy(self.http(), id.get())
    }

    fn dashboard_users_index(&self) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::users::dashboard_index(self.http())
    }

    fn dashboard_users_index_with_params(
        &self,
        params: Form,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        endpoints::users::dashboard_index_with_params(self.http(), params)
    }
}
