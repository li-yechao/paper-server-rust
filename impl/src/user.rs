use std::ops::Deref;

use async_trait::async_trait;
use bson::{doc, Document};
use lazy_static::lazy_static;
use mongodb::options::{FindOneAndUpdateOptions, FindOneOptions};
use paper::{user::*, Error, Result};
use shaku::Provider;

use crate::utils::*;

#[derive(Provider)]
#[shaku(interface = UserService)]
pub struct UserServiceImpl {
    #[shaku(provide)]
    user_collection: Box<dyn UserCollection>,
}

pub trait UserCollection: Deref<Target = mongodb::Collection> + Send + Sync {}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn select_user(&self, identifier: UserIdentifier) -> Result<User> {
        self.find_user(identifier).await
    }

    async fn create_user(&self, input: CreateUserInput) -> Result<User> {
        let id = new_id().into();
        let created_at = now_msec();

        let user = match input {
            CreateUserInput::Github { github_user } => User {
                id,
                created_at,
                name: github_user
                    .as_object()
                    .and_then(|obj| obj.get("name"))
                    .and_then(|name| name.as_str())
                    .ok_or_else(|| Error::unknown("Invalid github user name".to_owned()))?
                    .to_string(),
                github_user: Some(github_user),
                google_user: None,
            },
            CreateUserInput::Google { google_user } => User {
                id,
                created_at,
                name: google_user
                    .as_object()
                    .and_then(|obj| obj.get("name"))
                    .and_then(|name| name.as_str())
                    .ok_or_else(|| Error::unknown("Invalid google user name".to_owned()))?
                    .to_string(),
                github_user: None,
                google_user: Some(google_user),
            },
        };

        self.user_collection
            .insert_one(to_doc(&user)?, None)
            .await
            .map_err(|e| Error::unknown(e.to_string()))?;

        Ok(user)
    }

    async fn update_user(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        input: UpdateUserInput,
    ) -> Result<User> {
        if viewer_id != user_id {
            return Err(Error::forbidden(None));
        }

        let mut update_set = doc! {};

        if let Some(name) = input.name {
            update_set.insert("name", name);
        }

        if update_set.is_empty() {
            return self.select_user(UserIdentifier::Id(user_id)).await;
        }

        let options = FindOneAndUpdateOptions::builder()
            .return_document(mongodb::options::ReturnDocument::After)
            .projection(USER_PROJECTION.to_owned())
            .build();

        self.user_collection
            .find_one_and_update(
                doc! { "_id": user_id.to_string() },
                doc! { "$set": update_set },
                options,
            )
            .await
            .map_err(|e| Error::unknown(e.to_string()))?
            .map(|x| from_doc::<User>(x))
            .transpose()?
            .ok_or_else(|| Error::unknown("User not found".to_owned()))
    }

    async fn can_viewer_read_user(&self, viewer_id: UserId, user_id: UserId) -> Result<User> {
        let user = self.find_user(UserIdentifier::Id(user_id)).await?;

        if viewer_id == user.id {
            return Ok(user);
        }

        return Err(Error::forbidden(
            "You are not allowed to read this user".to_owned(),
        ));
    }

    async fn can_viewer_write_user(&self, viewer_id: UserId, user_id: UserId) -> Result<User> {
        let user = self.find_user(UserIdentifier::Id(user_id)).await?;

        if viewer_id == user.id {
            return Ok(user);
        }

        return Err(Error::forbidden(
            "You are not allowed to write this user".to_owned(),
        ));
    }

    async fn can_viewer_administer_user(&self, viewer_id: UserId, user_id: UserId) -> Result<User> {
        let user = self.find_user(UserIdentifier::Id(user_id)).await?;

        if viewer_id == user.id {
            return Ok(user);
        }

        return Err(Error::forbidden(
            "You are not allowed to administer this user".to_owned(),
        ));
    }
}

impl UserServiceImpl {
    async fn find_user(&self, identifier: UserIdentifier) -> Result<User> {
        let query = match &identifier {
            UserIdentifier::Id(id) => doc! { "_id": id.to_string() },
            UserIdentifier::Name(name) => doc! { "name": name },
            UserIdentifier::GithubUserId(gid) => doc! { "github_user.id": gid },
            UserIdentifier::GoogleUserId(gid) => doc! { "google_user.id": gid },
        };

        let user = self
            .user_collection
            .find_one(
                query,
                FindOneOptions::builder()
                    .projection(USER_PROJECTION.to_owned())
                    .build(),
            )
            .await
            .map_err(|e| Error::unknown(e.to_string()))?
            .map(|doc| from_doc::<User>(doc))
            .transpose()?;

        if user.is_some() {
            return Ok(user.unwrap());
        }

        Err(Error::not_found("User not found".to_owned()))
    }
}

lazy_static! {
    static ref USER_PROJECTION: Document = doc! {
        "_id": 1,
        "created_at": 1,
        "name": 1,
        "github_user": 1,
        "google_user": 1,
    };
}
