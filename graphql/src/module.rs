use paper_impl::{auth::*, paper::*, user::*};

crate::shaku_storage_collection_config!(
    UserCollectionConfigInterface,
    UserCollectionConfig,
    UserCollection,
    UserCollectionImpl
);

crate::shaku_storage_collection_config!(
    PaperCollectionConfigInterface,
    PaperCollectionConfig,
    PaperCollection,
    PaperCollectionImpl
);

shaku::module! {
    pub Module {
        components = [
            UserCollectionConfig,
            PaperCollectionConfig,

            AccessTokenConfig,
            RefreshTokenConfig,

            GithubAuthConfig,
            GoogleAuthConfig,
        ],
        providers = [
            UserCollectionImpl,
            PaperCollectionImpl,

            AuthServiceImpl,
            UserServiceImpl,
            PaperServiceImpl,
        ]
    }
}
