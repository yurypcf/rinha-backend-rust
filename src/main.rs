use std::{collections::HashMap, sync::Arc};

use axum::{
  routing::{get, post},
  Router,
  response::IntoResponse,
  http::StatusCode, extract::{State, Path}, Json,
};

use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use time::{Date, macros::date};

time::serde::format_description!(date_format, Date, "[year]-[month]-[day]");

#[derive(Serialize, Clone)]
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

#[derive(Deserialize, Clone)]
pub struct NewPersonName(String);

impl TryFrom<String> for NewPersonName {
  type Error = &'static str;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    if value.len() > 100 {
      Ok(NewPersonName(value))
    } else {
      Err("name is greater than permitted")
    }
  }
}

#[derive(Deserialize, Clone)]
pub struct NewPersonNick(String);

impl TryFrom<String> for NewPersonNick {
  type Error = &'static str;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    if value.len() > 32 {
      Ok(NewPersonNick(value))
    } else {
      Err("nick is greater than permitted")
    }
  }
}

#[derive(Deserialize, Clone)]
pub struct Tech(String);

impl TryFrom<String> for Tech {
  type Error = &'static str;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    if value.len() > 32 {
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

#[derive(Deserialize, Clone)]
pub struct NewPerson {
  #[serde(rename = "nome")]
  pub name: NewPersonName,
  #[serde(rename = "apelido")]
  pub nick: NewPersonNick,
  #[serde(rename = "nascimento", with = "date_format")]
  pub birth_date: Date,
  pub stack: Option<Vec<Tech>>,
}

type AppState = Arc<RwLock<HashMap<Uuid, Person>>>;

#[tokio::main]
async fn main() {

  let mut people: HashMap<Uuid, Person> = HashMap::new();

  let person = Person {
    id: Uuid::now_v7(),
    name: "Yury Cavalcante".to_string(),
    nick: "cavalcanteyury".to_string(),
    birth_date: date!(1997 - 01 - 20),
    stack: Some(vec!["Ruby".to_string(), "Rust".to_string()]),
  };

  println!("{:?}", &person.id);

  people.insert(person.id, person);

  let app_state: AppState = Arc::new(RwLock::new(people));

  // build our application with a single route
  let app = Router::new()
      .route("/pessoas", get(search_people))
      .route("/pessoas/:id", get(find_person))
      .route("/pessoas", post(create_person))
      .route("/contagem-pessoas", get(count_people))
      .with_state(app_state);


  // run it with hyper on localhost:3000
  axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
      .serve(app.into_make_service())
      .await
      .unwrap();
}

async fn search_people() -> impl IntoResponse {
  (StatusCode::NOT_FOUND, "Busca de Pessoas")
}

async fn find_person(
  State(people): State<AppState>,
  Path(person_id): Path<Uuid>,
) -> impl IntoResponse {
  match people.read().await.get(&person_id) {
    Some(person) => Ok(Json(person.clone())),
    None => Err(StatusCode::NOT_FOUND),
  }
}

async fn create_person(
  State(people): State<AppState>,
  Json(new_person): Json<NewPerson>
) -> impl IntoResponse {
  let id = Uuid::now_v7();

  let new_person: Person = Person {
    id,
    name: new_person.name.0,
    nick: new_person.nick.0,
    birth_date: new_person.birth_date,
    stack: new_person.stack.map(|stack: Vec<Tech>| stack.into_iter().map(String::from).collect())
  };

  people.write().await.insert(new_person.id, new_person.clone());

  (StatusCode::OK, Json(new_person))
}

async fn count_people() -> impl IntoResponse {
  (StatusCode::NOT_FOUND, "Contagem de pessoas")
}