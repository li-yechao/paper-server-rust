use std::{fs::File, io::Read};

use actix_web::{
    middleware::Condition, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use juniper::EmptySubscription;
use paper_graphql::{logger::Logger, models::paper::*, *};
use paper_impl::auth::*;

#[actix_web::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = build_config()?;
    let addr = (config.address.to_owned(), config.port);

    Logger::init(config.log_level).map_err(|e| e.to_string())?;

    let db = mongodb::Client::with_uri_str(&config.storage.uri)
        .await?
        .database(&config.storage.database);

    let _ = HttpServer::new(move || {
        let module = Module::builder()
            .with_component_parameters::<AccessTokenConfig>(AccessTokenConfigParameters {
                expires_in_sec: config.access_token.expires_in_sec,
                secret: config.access_token.secret.to_owned(),
            })
            .with_component_parameters::<RefreshTokenConfig>(RefreshTokenConfigParameters {
                expires_in_sec: config.refresh_token.expires_in_sec,
                secret: config.refresh_token.secret.to_owned(),
            })
            .with_component_parameters::<PaperTokenConfig>(PaperTokenConfigParameters {
                expires_in_sec: config.paper_token.expires_in_sec,
                secret: config.paper_token.secret.to_owned(),
            })
            .with_component_parameters::<GithubAuthConfig>(GithubAuthConfigParameters {
                list: config
                    .github_auth
                    .iter()
                    .map(|x| GithubAuthConfigItem {
                        client_id: x.client_id.to_owned(),
                        client_secret: x.client_secret.to_owned(),
                    })
                    .collect(),
            })
            .with_component_parameters::<GoogleAuthConfig>(GoogleAuthConfigParameters {
                list: config
                    .google_auth
                    .iter()
                    .map(|x| GoogleAuthConfigItem {
                        client_id: x.client_id.to_owned(),
                        client_secret: x.client_secret.to_owned(),
                        redirect_uri: x.redirect_uri.to_owned(),
                    })
                    .collect(),
            })
            .with_component_parameters::<UserCollectionConfig>(UserCollectionConfigParameters {
                database: db.clone(),
                collection: config.storage.collection_user.to_owned(),
            })
            .with_component_parameters::<PaperCollectionConfig>(PaperCollectionConfigParameters {
                database: db.clone(),
                collection: config.storage.collection_paper.to_owned(),
            })
            .build();

        App::new()
            .wrap(Condition::new(
                config.cors,
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            ))
            .data(module)
            .data(Schema::new(Query, Mutation, EmptySubscription::new()))
            .service(graphql_handler)
            .service(graphiql_handler)
    })
    .bind(addr)
    .unwrap()
    .run()
    .await;

    Ok(())
}

#[actix_web::get("/graphiql")]
async fn graphiql_handler() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("../../graphiql.html"))
}

#[actix_web::post("/graphql")]
async fn graphql_handler(
    req: HttpRequest,
    payload: web::Payload,
    schema: web::Data<Schema>,
    module: web::Data<Module>,
) -> impl Responder {
    let access_token = req
        .headers()
        .get("Authorization")
        .and_then(|x| x.to_str().ok())
        .and_then(|x| {
            if x.starts_with("Bearer ") {
                Some(x.trim_start_matches("Bearer ").to_owned())
            } else {
                None
            }
        });

    let context = Context {
        module: module.into_inner(),
        access_token,
    };

    juniper_actix::graphql_handler(&schema, &context, req, payload).await
}

fn build_config() -> std::result::Result<Config, Box<dyn std::error::Error>> {
    let matches = clap::App::new(env!("CARGO_BIN_NAME"))
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            clap::Arg::with_name("config")
                .short("c")
                .long("config")
                .help("path to configuration file")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let config_file = matches.value_of("config").expect("config file is required");
    let mut file = File::open(config_file)?;
    let mut conf = String::new();
    file.read_to_string(&mut conf)?;
    Ok(toml::from_str(&conf)?)
}
