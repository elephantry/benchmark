mod user {
    #[derive(Clone, elephantry::Entity)]
    pub struct Entity {
        pub id: Option<i32>,
        pub name: String,
        pub hair_color: Option<String>,
        pub created_at: Option<chrono::NaiveDateTime>,
        #[elephantry(default)]
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
    }

    pub struct Model<'a> {
        connection: &'a elephantry::Connection,
    }

    impl<'a> elephantry::Model<'a> for Model<'a> {
        type Entity = Entity;
        type Structure = Structure;

        fn new(connection: &'a elephantry::Connection) -> Self {
            Self {
                connection,
            }
        }
    }

    impl<'a> Model<'a> {
        pub fn user_with_posts(&self, id: i32) -> Result<Entity, elephantry::Error> {
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

    pub struct Structure;

    impl elephantry::Structure for Structure {
        fn relation() -> &'static str {
            "public.users"
        }

        fn primary_key() -> &'static [&'static str] {
            &["id"]
        }

        fn columns() -> &'static [&'static str] {
            &["id", "name", "hair_color", "created_at"]
        }
    }
}

impl crate::Client for elephantry::Pool {
    type Error = elephantry::Error;
    type User = user::Entity;

    fn create(dsn: &str) -> Result<Self, Self::Error> {
        elephantry::Pool::new(dsn)
    }

    fn exec(&mut self, query: &str) -> Result<(), Self::Error> {
        self.execute(&query).map(|_| ())
    }

    fn insert_user(&mut self) -> Result<(), Self::Error> {
        self.insert_one::<user::Model>(&user::Entity::new())
            .map(|_| ())
    }

    fn fetch_all(&mut self) -> Result<Vec<Self::User>, Self::Error> {
        let results = self
            .find_all::<user::Model>(None)?
            .collect::<Vec<Self::User>>();

        Ok(results)
    }

    fn fetch_first(&mut self) -> Result<Self::User, Self::Error> {
        let result = self.find_all::<user::Model>(None)?.next();

        Ok(result.unwrap())
    }

    fn fetch_last(&mut self) -> Result<Self::User, Self::Error> {
        let result = self.find_all::<user::Model>(None)?.get(9_999);

        Ok(result)
    }

    fn one_relation(&mut self) -> Result<(Self::User, Vec<String>), Self::Error> {
        let user = self.model::<user::Model>()
            .user_with_posts(42)?;
        let posts = user.posts.clone();

        Ok((user, posts))
    }

    fn all_relations(&mut self) -> Result<Vec<(Self::User, Vec<String>)>, Self::Error> {
        let users = self.model::<user::Model>()
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

crate::bench! {elephantry::Pool}
