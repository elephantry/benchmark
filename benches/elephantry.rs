mod user {
    #[derive(elephantry::Entity)]
    pub struct Entity {
        pub id: Option<i32>,
        pub name: String,
        pub hair_color: Option<String>,
        pub created_at: Option<chrono::NaiveDateTime>,
    }

    impl Entity {
        pub fn new(x: usize) -> Self {
            Self {
                id: None,
                name: format!("User {}", x),
                hair_color: Some(format!("hair color {}", x)),
                created_at: None,
            }
        }
    }

    pub struct Model;

    impl<'a> elephantry::Model<'a> for Model {
        type Entity = Entity;
        type Structure = Structure;

        fn new(_: &'a elephantry::Connection) -> Self {
            Self {}
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

        fn definition() -> &'static [&'static str] {
            &["id", "name", "hair_color", "created_at"]
        }
    }
}

impl crate::Client for elephantry::Pool {
    type Entity = user::Entity;
    type Error = elephantry::Error;

    fn create(dsn: &str) -> Result<Self, Self::Error> {
        elephantry::Pool::new(dsn)
    }

    fn exec(&mut self, query: &str) -> Result<(), Self::Error> {
        self.execute(&query).map(|_| ())
    }

    fn insert_x(&mut self, x: usize) -> Result<(), Self::Error> {
        self.insert_one::<user::Model>(&user::Entity::new(x))
            .map(|_| ())
    }

    fn fetch_all(&mut self) -> Result<Vec<Self::Entity>, Self::Error> {
        let results = self
            .find_all::<user::Model>(None)?
            .collect::<Vec<Self::Entity>>();

        Ok(results)
    }

    fn fetch_first(&mut self) -> Result<Self::Entity, Self::Error> {
        let result = self.find_all::<user::Model>(None)?.next();

        Ok(result.unwrap())
    }

    fn fetch_last(&mut self) -> Result<Self::Entity, Self::Error> {
        let result = self.find_all::<user::Model>(None)?.get(9_999);

        Ok(result)
    }
}

crate::bench! {elephantry::Pool}
