use std::{sync::Arc, env, net::SocketAddr};
use axum::{
  routing::{get, post},
  Router,
  response::IntoResponse,
  http::StatusCode, extract::{State, Path, Query}, Json,
};
use persistence::PostgresRepository;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use time::Date;

mod persistence;

time::serde::format_description!(date_format, Date, "[year]-[month]-[day]");

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct Person {
  pub id: Uuid,
  #[serde(rename = "nome")]
  pub name: String,
  #[serde(rename = "apelido")]
  pub nick: String,
  #[serde(rename = "nascimento", with = "date_format")]
  pub birth_date: Date,
  pub stack: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewPerson {
  #[serde(rename = "nome")]
  pub name: NewPersonName,
  #[serde(rename = "apelido")]
  pub nick: NewPersonNick,
  #[serde(rename = "nascimento", with = "date_format")]
  pub birth_date: Date,
  pub stack: Option<Vec<Tech>>,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct NewPersonName(String);

impl TryFrom<String> for NewPersonName {
  type Error = &'static str;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    if value.len() <= 100 {
      Ok(NewPersonName(value))
    } else {
      Err("name is greater than permitted")
    }
  }
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct NewPersonNick(String);

impl TryFrom<String> for NewPersonNick {
  type Error = &'static str;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    if value.len() <= 32 {
      Ok(NewPersonNick(value))
    } else {
      Err("nick is greater than permitted")
    }
  }
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Tech(String);

impl TryFrom<String> for Tech {
  type Error = &'static str;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    if value.len() <= 32 {
      Ok(Tech(value))
    } else {
      Err("tech is greater than permitted")
    }
  }
}

impl From<Tech> for String {
  fn from(value: Tech) -> Self {
      value.0
  }
}

#[derive(Deserialize)]
struct PersonSearchQuery {
  #[serde(rename = "t")]
  query: String,
}

type AppState = Arc<PostgresRepository>;

#[tokio::main]
async fn main() {

  let port = env::var("PORT")
    .ok()
    .and_then(|port| port.parse::<u16>().ok())
    .unwrap_or(9999);

  let database_url = env::var("DATABASE_URL")
    .unwrap_or(String::from("postgres://rinha:rinha@localhost:5432/rinha"));

  let postgres_repo = persistence::PostgresRepository::connect(database_url).await;

  let app_state: AppState = Arc::new(postgres_repo);

  // build our application with a single route
  let app = Router::new()
    .route("/pessoas", get(search_people))
    .route("/pessoas/:id", get(find_person))
    .route("/pessoas", post(create_person))
    .route("/contagem-pessoas", get(count_people))
    .with_state(app_state);


  // run it with hyper on localhost:3000
  axum::Server::bind(&SocketAddr::from(([0, 0, 0, 0], port)))
    .serve(app.into_make_service())
    .await
    .unwrap();
}

async fn search_people(
  State(people): State<AppState>,
  Query(PersonSearchQuery { query }): Query<PersonSearchQuery>,
) -> impl IntoResponse {
  match people.search_people(query).await {
    Ok(people) => Ok(Json(people.clone())),
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}

async fn find_person(
  State(people): State<AppState>,
  Path(person_id): Path<Uuid>,
) -> impl IntoResponse {
  match people.find_person(person_id).await {
    Ok(Some(person)) => Ok(Json(person)),
    Ok(None) => Err(StatusCode::NOT_FOUND),
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
}
}

async fn create_person(
  State(people): State<AppState>,
  Json(new_person): Json<NewPerson>
) -> impl IntoResponse {
  match people.create_person(new_person).await {
    Ok(person) => Ok((StatusCode::CREATED, Json(person))),
    Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
        Err(StatusCode::UNPROCESSABLE_ENTITY)
    }
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}

async fn count_people(State(people): State<AppState>) -> impl IntoResponse {
  match people.count_people().await {
    Ok(count) => Ok(Json(count)),
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}