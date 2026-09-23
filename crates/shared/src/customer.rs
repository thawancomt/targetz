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
    pub created_at: String,
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
            created_at: String::new(),
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
            created_at: self.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct Stakeholder {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub phone_number: String,
    pub address: Option<String>,
    pub instagram_url: Option<String>,
    pub site_url: Option<String>,
    pub is_client: bool,
    pub contacted: bool,
    pub created_at: String,
    pub role: StakeholderRole,
}

impl Stakeholder {
    pub fn customer(self) -> Customer {
        Customer {
            id: self.id,
            name: self.name,
            email: self.email,
            phone_number: self.phone_number,
            address: self.address,
            instagram_url: self.instagram_url,
            site_url: self.site_url,
            is_client: self.is_client,
            contacted: self.contacted,
            created_at: self.created_at,
        }
    }
}

impl Stakeholder {
    pub fn new(customer: Customer, role: Option<StakeholderRole>) -> Self {
        Self {
            id: customer.id,
            name: customer.name,
            email: customer.email,
            phone_number: customer.phone_number,
            address: customer.address,
            instagram_url: customer.instagram_url,
            site_url: customer.site_url,
            is_client: customer.is_client,
            contacted: customer.contacted,
            role: role.unwrap_or(StakeholderRole::Partner),
            created_at: customer.created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StakeholderRole {
    Owner,
    Investor,
    Partner,
}

impl From<String> for StakeholderRole {
    fn from(value: String) -> Self {
        match value.as_str() {
            "owner" => Self::Owner,
            "investor" => Self::Investor,
            "partner" => Self::Partner,
            _ => Self::Partner,
        }
    }
}

impl StakeholderRole {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Investor => "investor",
            Self::Owner => "owner",
            Self::Partner => "partner",
        }
    }
}
