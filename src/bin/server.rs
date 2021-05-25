use grpc_address_book::address_book::{
    address_book_server::{AddressBook, AddressBookServer},
    *,
};

use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new().connect(&database_url).await?;
    sqlx::migrate!().run(&pool).await?;

    let service = AddressBookService { pool };
    let server = AddressBookServer::new(service);
    let addr = std::env::var("SERVER_ADDR")?.parse()?;
    Server::builder().add_service(server).serve(addr).await?;

    Ok(())
}

struct AddressBookService {
    pool: PgPool,
}

#[async_trait]
impl AddressBook for AddressBookService {
    async fn create_person(
        &self,
        request: Request<CreatePersonRequest>,
    ) -> Result<Response<CreatePersonResponse>, Status> {
        let CreatePersonRequest { name, email } = request.into_inner();
        let result = sqlx::query!(
            r#"
INSERT INTO people (name, email)
VALUES ($1, $2)
RETURNING uuid"#,
            name,
            email
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(CreatePersonResponse {
            uuid: result.uuid.to_string(),
        }))
    }

    async fn get_person(
        &self,
        request: Request<GetPersonRequest>,
    ) -> Result<Response<GetPersonResponse>, Status> {
        let GetPersonRequest { uuid } = request.into_inner();
        let uuid = uuid
            .parse::<Uuid>()
            .map_err(|e| Status::internal(e.to_string()))?;
        let person = sqlx::query_as!(
            DbPerson,
            r#"
SELECT uuid, name, email
FROM people
WHERE uuid = $1;"#,
            uuid
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::not_found("Person not found"))?
        .into();

        Ok(Response::new(GetPersonResponse {
            person: Some(person),
        }))
    }

    async fn list_people(
        &self,
        _request: Request<()>,
    ) -> Result<Response<ListPeopleResponse>, Status> {
        let people = sqlx::query_as!(
            DbPerson,
            r#"
SELECT uuid, name, email
FROM people;"#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .into_iter()
        .map(Into::into)
        .collect();
        Ok(Response::new(ListPeopleResponse { people }))
    }

    async fn search_people(
        &self,
        request: Request<SearchPeopleRequest>,
    ) -> Result<Response<SearchPeopleResponse>, Status> {
        let SearchPeopleRequest { name, email } = request.into_inner();
        let people = sqlx::query_as!(
            DbPerson,
            r#"
SELECT uuid, name, email
FROM people
WHERE
    (name IS NOT NULL AND name ILIKE $1) OR
    (email IS NOT NULL AND email ILIKE $2);"#,
            name.map(|name| format!("%{}%", name)),
            email.map(|email| format!("%{}%", email)),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .into_iter()
        .map(Into::into)
        .collect();
        Ok(Response::new(SearchPeopleResponse { people }))
    }

    async fn update_person(
        &self,
        request: Request<UpdatePersonRequest>,
    ) -> Result<Response<()>, Status> {
        let UpdatePersonRequest { uuid, name, email } = request.into_inner();
        let uuid = uuid
            .parse::<Uuid>()
            .map_err(|e| Status::internal(e.to_string()))?;
        let result = sqlx::query!(
            r#"
UPDATE people
SET
    name = COALESCE($1, name),
    email = COALESCE($2, email)
WHERE uuid = $3"#,
            name,
            email,
            uuid
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(Status::not_found("Person not found"));
        }

        Ok(Response::new(()))
    }

    async fn delete_person(
        &self,
        request: Request<DeletePersonRequest>,
    ) -> Result<Response<()>, Status> {
        let DeletePersonRequest { uuid } = request.into_inner();
        let uuid = uuid
            .parse::<Uuid>()
            .map_err(|e| Status::internal(e.to_string()))?;
        let result = sqlx::query!("DELETE FROM people WHERE uuid = $1", uuid)
            .execute(&self.pool)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(Status::not_found("Person not found"));
        }

        Ok(Response::new(()))
    }
}

struct DbPerson {
    uuid: Uuid,
    name: String,
    email: Option<String>,
}

impl From<DbPerson> for Person {
    fn from(p: DbPerson) -> Self {
        Self {
            uuid: p.uuid.to_string(),
            name: p.name,
            email: p.email,
        }
    }
}
