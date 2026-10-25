use sabai::DateTime;
use sabai::auth::Authenticatable;
use sabai::orm::Model;
use serde::Serialize;

/// Someone who can sign in, like Laravel's `App\Models\User`.
#[derive(Default, Serialize, Model)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
}

impl Authenticatable for User {
    fn password(&self) -> &str {
        &self.password
    }

    fn set_password(&mut self, hashed: String) {
        self.password = hashed;
    }
}
