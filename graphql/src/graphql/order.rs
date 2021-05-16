use juniper::GraphQLEnum;

#[derive(GraphQLEnum)]
pub enum OrderDirection {
    #[graphql(name = "ASC")]
    Asc,

    #[graphql(name = "DESC")]
    Desc,
}

impl Into<paper::OrderDirection> for OrderDirection {
    fn into(self) -> paper::OrderDirection {
        match self {
            OrderDirection::Asc => paper::OrderDirection::Asc,
            OrderDirection::Desc => paper::OrderDirection::Desc,
        }
    }
}
