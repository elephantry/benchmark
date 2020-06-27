use diesel::prelude::*;

#[derive(diesel::Insertable)]
#[table_name = "users"]
pub struct NewUser {
    name: String,
    hair_color: Option<String>,
}

impl NewUser {
    pub fn new(x: usize) -> Self {
        NewUser {
            name: format!("User {}", x),
            hair_color: Some(format!("hair color {}", x)),
        }
    }
}

diesel::table! {
    users {
        id -> Serial,
        name -> VarChar,
        hair_color -> Nullable<VarChar>,
        created_at -> Timestamp,
    }
}

#[derive(Clone, diesel::Queryable)]
pub struct User {
    id: i32,
    name: String,
    hair_color: Option<String>,
    created_at: chrono::NaiveDateTime,
}

impl crate::Client for diesel::pg::PgConnection {
    type Entity = User;
    type Error = diesel::result::Error;

    fn create(dsn: &str) -> Result<Self, Self::Error> {
        let client = diesel::pg::PgConnection::establish(dsn).unwrap();

        Ok(client)
    }

    fn exec(&mut self, query: &str) -> Result<(), Self::Error> {
        self.execute(query).map(|_| ())
    }

    fn tear_down(&mut self) -> Result<(), Self::Error> {
        self.execute("DROP TABLE users;").map(|_| ())
    }

    fn insert_x(&mut self, x: usize) -> Result<(), Self::Error> {
        diesel::insert_into(users::table)
            .values(&NewUser::new(x))
            .execute(self)
            .map(|_| ())
    }

    fn insert(&mut self, n: usize) -> Result<(), Self::Error> {
        let vals = (0..n).map(NewUser::new).collect::<Vec<_>>();
        diesel::insert_into(users::table)
            .values(&vals)
            .execute(self)
            .map(|_| ())
    }

    fn fetch_all(&mut self) -> Result<Vec<Self::Entity>, Self::Error> {
        users::table.load::<Self::Entity>(self)
    }

    fn fetch_first(&mut self) -> Result<Self::Entity, Self::Error> {
        let results = users::table.load::<User>(self)?;

        Ok(results.first().unwrap().clone())
    }

    fn fetch_last(&mut self) -> Result<Self::Entity, Self::Error> {
        let results = users::table.load::<User>(self)?;

        Ok(results[9_999].clone())
    }
}

crate::bench! {diesel::pg::PgConnection}
