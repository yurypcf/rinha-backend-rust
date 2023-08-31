use std::{collections::HashMap, sync::Arc};

use axum::{
  routing::{get, post},
  Router,
  response::IntoResponse,
  http::StatusCode, extract::{State, Path}, Json,
};
use serde::Serialize;
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

type AppState = Arc<HashMap<Uuid, Person>>;

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

  let app_state: AppState = Arc::new(people);

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
  match people.get(&person_id) {
    Some(person) => Ok(Json(person.clone())),
    None => Err(StatusCode::NOT_FOUND),
  }
}

async fn create_person() -> impl IntoResponse {
  (StatusCode::NOT_FOUND, "Criacao de uma pessoa")
}

async fn count_people() -> impl IntoResponse {
  (StatusCode::NOT_FOUND, "Contagem de pessoas")
}