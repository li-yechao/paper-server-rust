mod config;
mod context;
mod graphql;
mod module;
mod mutation;
mod query;

pub mod logger;
pub mod models;

pub use config::*;
pub use context::{Context, Schema};
pub use graphql::*;
pub use module::*;
pub use mutation::Mutation;
pub use query::Query;

#[macro_export]
macro_rules! shaku_storage_collection_config {
    ($config_if:ident, $config_impl:ident, $collection_if:path, $collection_impl:ident) => {
        #[derive(Debug, shaku::Component)]
        #[shaku(interface = $config_if)]
        pub struct $config_impl {
            pub database: mongodb::Database,

            pub collection: String,
        }

        pub trait $config_if: shaku::Interface {
            fn collection(&self) -> mongodb::Collection;
        }

        impl $config_if for $config_impl {
            fn collection(&self) -> mongodb::Collection {
                self.database.collection(&self.collection)
            }
        }

        #[derive(Debug)]
        pub struct $collection_impl(mongodb::Collection);

        impl std::ops::Deref for $collection_impl {
            type Target = mongodb::Collection;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl $collection_if for $collection_impl {}

        impl<M> shaku::Provider<M> for $collection_impl
        where
            M: shaku::Module + shaku::HasComponent<dyn $config_if>,
        {
            type Interface = dyn $collection_if;

            fn provide(
                module: &M,
            ) -> Result<Box<dyn $collection_if>, Box<dyn std::error::Error + 'static>> {
                let db: &dyn $config_if = module.resolve_ref();
                Ok(Box::new(Self(db.collection())))
            }
        }
    };
}
