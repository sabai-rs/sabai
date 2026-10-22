use sabai::auth::Authenticatable;
use sabai::orm::Model;
use serde::Serialize;

/// Someone who can sign in, like Laravel's `App\Models\User`.
#[derive(Serialize, Model)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
}

impl Authenticatable for User {
    fn password(&self) -> &str {
        &self.password
    }

    fn set_password(&mut self, hashed: String) {
        self.password = hashed;
    }
}
