use sqlx::prelude::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Draft;

pub type Persisted = i64;

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct Customer<Id = Persisted> {
    pub id: Id,
    pub name: String,
    pub email: String,
    pub phone_number: String,
    pub address: Option<String>,
    pub instagram_url: Option<String>,
    pub site_url: Option<String>,
    pub is_client: bool,
    pub contacted: bool,
}

// 3. Métodos para Customer<Draft> (sem ID no banco ainda)
impl Customer<Draft> {
    pub fn new(
        name: impl Into<String>,
        email: impl Into<String>,
        phone_number: impl Into<String>,
    ) -> Self {
        Self {
            id: Draft,
            name: name.into(),
            email: email.into(),
            phone_number: phone_number.into(),
            address: None,
            instagram_url: None,
            site_url: None,
            is_client: false,
            contacted: false,
        }
    }

    pub fn into_persisted(self, id: Persisted) -> Customer<Persisted> {
        Customer {
            id,
            name: self.name,
            email: self.email,
            phone_number: self.phone_number,
            address: self.address,
            instagram_url: self.instagram_url,
            site_url: self.site_url,
            is_client: self.is_client,
            contacted: self.contacted,
        }
    }
}
