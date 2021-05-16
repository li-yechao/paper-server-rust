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

crate::shaku_storage_collection_config!(
    PaperContentCollectionConfigInterface,
    PaperContentCollectionConfig,
    PaperContentCollection,
    PaperContentCollectionImpl
);

shaku::module! {
    pub Module {
        components = [
            UserCollectionConfig,
            PaperCollectionConfig,
            PaperContentCollectionConfig,

            AccessTokenConfig,
            RefreshTokenConfig,

            GithubAuthConfig,
            GoogleAuthConfig,
        ],
        providers = [
            UserCollectionImpl,
            PaperCollectionImpl,
            PaperContentCollectionImpl,


            AuthServiceImpl,
            UserServiceImpl,
            PaperServiceImpl,
        ]
    }
}
