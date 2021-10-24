#![feature(test)]
#![allow(soft_unstable)]

extern crate test;

mod user {
    #[derive(Clone, elephantry::Entity)]
    #[elephantry(model = "Model", structure = "Structure", relation = "public.users")]
    pub struct Entity {
        #[elephantry(pk)]
        pub id: Option<uuid::Uuid>,
        pub name: String,
        pub hair_color: Option<String>,
        pub created_at: Option<chrono::NaiveDateTime>,
        #[elephantry(default, virtual)]
        pub posts: Vec<String>,
    }

    impl Entity {
        pub fn new() -> Self {
            Self {
                id: None,
                name: format!("User"),
                hair_color: Some(format!("hair color")),
                created_at: None,
                posts: Vec::new(),
            }
        }

        pub fn with_id(id: uuid::Uuid) -> Self {
            Self {
                id: Some(id),
                name: format!("User"),
                hair_color: Some(format!("hair color")),
                created_at: Some(chrono::offset::Local::now().naive_local()),
                posts: Vec::new(),
            }
        }
    }

    impl<'a> Model<'a> {
        pub fn user_with_posts(&self, id: uuid::Uuid) -> Result<Entity, elephantry::Error> {
            use elephantry::Model;

            let query = r#"
select {projection}
    from users u
    join posts p on p.author = u.id
    where u.id = $1
    group by u.id, u.name, u.hair_color, u.created_at
"#;

            let projection = Self::create_projection()
                .alias("u")
                .add_field("posts", "array_agg(p.title)");

            let sql = query.replace("{projection}", &projection.to_string());

            Ok(self.connection.query::<Entity>(&sql, &[&id])?.get(0))
        }

        pub fn users_with_posts(&self) -> Result<Vec<Entity>, elephantry::Error> {
            use elephantry::Model;

            let query = r#"
select {projection}
    from users u
    join posts p on p.author = u.id
    group by u.id, u.name, u.hair_color, u.created_at
"#;

            let projection = Self::create_projection()
                .alias("u")
                .add_field("posts", "array_agg(p.title)");

            let sql = query.replace("{projection}", &projection.to_string());

            Ok(self.connection.query::<Entity>(&sql, &[])?.collect())
        }
    }
}

struct Connection(elephantry::Pool);

impl elephantry_benchmark::Client for Connection {
    type Error = elephantry::Error;
    type User = user::Entity;

    fn create(dsn: &str) -> Result<Self, Self::Error> {
        elephantry::Pool::new(dsn)
            .map(Self)
    }

    fn exec(&mut self, query: &str) -> Result<(), Self::Error> {
        self.0.execute(&query).map(|_| ())
    }

    fn insert_user(&mut self) -> Result<(), Self::Error> {
        self.0.insert_one::<user::Model>(&user::Entity::new())
            .map(|_| ())
    }

    fn insert_users(&mut self, n: usize) -> Result<(), Self::Error> {
        let users = (0..n).map(|_| user::Entity::with_id(uuid::Uuid::new_v4()));

        self.0.copy::<user::Model, _>(users)
    }

    fn fetch_all(&mut self) -> Result<Vec<Self::User>, Self::Error> {
        let results = self.0
            .find_all::<user::Model>(None)?
            .collect::<Vec<Self::User>>();

        Ok(results)
    }

    fn fetch_first(&mut self) -> Result<Self::User, Self::Error> {
        let result = self.0.find_all::<user::Model>(None)?.next();

        Ok(result.unwrap())
    }

    fn fetch_last(&mut self) -> Result<Self::User, Self::Error> {
        let result = self.0.find_all::<user::Model>(None)?.get(9_999);

        Ok(result)
    }

    fn one_relation(&mut self) -> Result<(Self::User, Vec<String>), Self::Error> {
        let user = self.0.model::<user::Model>()
            .user_with_posts(elephantry_benchmark::UUID)?;
        let posts = user.posts.clone();

        Ok((user, posts))
    }

    fn all_relations(&mut self) -> Result<Vec<(Self::User, Vec<String>)>, Self::Error> {
        let users = self.0.model::<user::Model>()
            .users_with_posts()?;

        Ok(users
            .iter()
            .map(|x| {
                let posts= x.posts.clone();
                ((*x).clone(), posts)
            })
            . collect()
        )
    }
}

elephantry_benchmark::bench! {Connection}
