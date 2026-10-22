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
