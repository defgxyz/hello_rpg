mod scene;
mod world;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Form, State,
    },
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::services::ServeDir;
use uuid::Uuid;
use world::{World, DIR_DOWN, DIR_LEFT, DIR_RIGHT, DIR_UP, MAX_X, MAX_Y};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scenes_dir = std::env::var("SCENES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("scenes"));
    let static_dir = std::env::var("STATIC_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("static"));

    let world = Arc::new(World::load(&scenes_dir));
    tracing::info!(
        "loaded {} scene(s) from {:?}",
        world.scenes.len(),
        scenes_dir
    );

    let app = Router::new()
        .route("/", get(index))
        .route("/login", post(login))
        .route("/game", get(game))
        .route("/ws", get(ws_handler))
        .nest_service("/assets", ServeDir::new(static_dir))
        .with_state(world);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html>
<head><title>Hello-RPG</title></head>
<body>
  <form method="post" action="/login">
    <input type="text" name="name" placeholder="name" required>
    <input type="text" name="hero" placeholder="hero" required>
    <input type="submit" value="Play">
  </form>
</body>
</html>"#,
    )
}

#[derive(Deserialize)]
struct LoginForm {
    name: String,
    hero: String,
}

async fn login(jar: CookieJar, Form(form): Form<LoginForm>) -> impl IntoResponse {
    let uuid = Uuid::new_v4().to_string();
    let jar = jar
        .add(Cookie::new("name", form.name))
        .add(Cookie::new("hero", form.hero))
        .add(Cookie::new("uuid", uuid));
    (jar, Redirect::to("/game"))
}

async fn game(jar: CookieJar) -> impl IntoResponse {
    if jar.get("name").is_none() || jar.get("uuid").is_none() {
        return Redirect::to("/").into_response();
    }

    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <title>Hello-RPG</title>
  <script src="/assets/javascripts/jquery.min.js"></script>
  <script src="/assets/javascripts/crafty.js"></script>
  <script src="/assets/javascripts/game.js"></script>
  <script src="/assets/javascripts/components.js"></script>
  <script src="/assets/javascripts/scenes.js"></script>
</head>
<body data-ws-url="{ws_url}">
  <div id="cr-stage" style="width: 384px; height: 256px;"></div>
</body>
</html>"#,
        ws_url = "/ws"
    ))
    .into_response()
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(world): State<Arc<World>>,
    jar: CookieJar,
) -> impl IntoResponse {
    let name = jar
        .get("name")
        .map(|c| c.value().to_string())
        .unwrap_or_default();
    let id = jar
        .get("uuid")
        .map(|c| c.value().to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    ws.on_upgrade(move |socket| handle_socket(socket, world, id, name))
}

async fn handle_socket(mut socket: WebSocket, world: Arc<World>, id: String, name: String) {
    let mut current_scene = world.default_scene_name();
    let Some(entry) = world.scenes.get(&current_scene) else {
        tracing::error!("no scenes available; closing connection for {}", id);
        let _ = socket.close().await;
        return;
    };
    let mut rx = entry.tx.subscribe();

    tracing::info!("{} ({}) connected, scene={}", name, id, current_scene);

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                let Some(incoming) = incoming else { break };
                let Ok(msg) = incoming else { break };
                match msg {
                    Message::Text(text) => {
                        handle_client_message(
                            &text,
                            &world,
                            &mut socket,
                            &mut current_scene,
                            &mut rx,
                            &id,
                            &name,
                        )
                        .await;
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            event = rx.recv() => {
                match event {
                    Ok((origin, out_event)) if origin != id => {
                        let payload = serde_json::to_string(&out_event).unwrap();
                        if socket.send(Message::Text(payload)).await.is_err() {
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
        }
    }

    tracing::info!("{} ({}) disconnected", name, id);
}

async fn send_load_scene(world: &World, socket: &mut WebSocket, scene_name: &str) -> bool {
    let Some(entry) = world.scenes.get(scene_name) else {
        return false;
    };
    let payload = serde_json::json!({ "type": "loadScene", "scene": entry.data.scene });
    socket.send(Message::Text(payload.to_string())).await.is_ok()
}

async fn handle_client_message(
    text: &str,
    world: &Arc<World>,
    socket: &mut WebSocket,
    current_scene: &mut String,
    rx: &mut tokio::sync::broadcast::Receiver<(String, world::OutEvent)>,
    id: &str,
    name: &str,
) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return;
    };
    let Some(msg_type) = value.get("type").and_then(|t| t.as_str()) else {
        return;
    };

    match msg_type {
        "subscribe" => {
            send_load_scene(world, socket, current_scene).await;
        }
        "userMove" => {
            let x = value.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let y = value.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

            let in_bounds = (0..=MAX_X).contains(&x) && (0..=MAX_Y).contains(&y);
            if in_bounds {
                if let Some(entry) = world.scenes.get(current_scene.as_str()) {
                    let _ = entry.tx.send((
                        id.to_string(),
                        world::OutEvent::UserMove {
                            id: id.to_string(),
                            name: name.to_string(),
                            x,
                            y,
                        },
                    ));
                }
                return;
            }

            let direction = if y < 0 {
                DIR_UP
            } else if y > MAX_Y {
                DIR_DOWN
            } else if x < 0 {
                DIR_LEFT
            } else {
                DIR_RIGHT
            };

            let target = world.transition_target(current_scene, direction);
            if target != *current_scene {
                if let Some(entry) = world.scenes.get(&target) {
                    *current_scene = target;
                    *rx = entry.tx.subscribe();
                    send_load_scene(world, socket, current_scene).await;
                }
            }
        }
        _ => {}
    }
}
