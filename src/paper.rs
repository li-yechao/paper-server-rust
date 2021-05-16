use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{user::UserId, Id, OrderBy, Pagination, PaginationList, Result};

#[async_trait]
pub trait PaperService: Send + Sync {
    async fn create_paper(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        input: CreatePaperInput,
    ) -> Result<(Paper, PaperContent)>;

    async fn update_paper(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        paper_id: PaperId,
        input: UpdatePaperInput,
    ) -> Result<(Paper, PaperContent)>;

    async fn delete_paper(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        paper_id: PaperId,
    ) -> Result<()>;

    async fn select_paper(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        paper_id: PaperId,
    ) -> Result<Paper>;

    async fn select_paper_content(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        paper_id: PaperId,
    ) -> Result<PaperContent>;

    async fn select_paper_page_of_repository(
        &self,
        viewer_id: UserId,
        user_id: UserId,
        pagination: Pagination<PaperId>,
        order_by: OrderBy<PaperOrderField>,
        deleted: bool,
    ) -> Result<PaginationList<Paper>>;

    async fn can_viewer_read_paper(
        &self,
        viewer_id: UserId,
        owner_id: UserId,
        paper_id: PaperId,
    ) -> Result<Paper>;

    async fn can_viewer_write_paper(
        &self,
        viewer_id: UserId,
        owner_id: UserId,
        paper_id: PaperId,
    ) -> Result<Paper>;
}

pub enum PaperOrderField {
    Id,

    UpdatedAt,
}

pub type PaperId = Id<Paper>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatePaperInput {
    pub title: Option<String>,

    pub content: Option<Vec<content::Block>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePaperInput {
    pub title: Option<String>,

    pub content: Option<Vec<content::Block>>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Paper {
    pub id: PaperId,

    pub user_id: UserId,

    pub created_at: u64,

    pub updated_at: u64,

    pub deleted_at: Option<u64>,

    pub title: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaperContent {
    pub id: PaperId,

    pub content: Option<Vec<content::Block>>,
}

pub mod content {
    use serde::{Deserialize, Serialize};
    use serde_repr::{Deserialize_repr, Serialize_repr};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Block {
        Heading(Heading),
        Paragraph(Paragraph),
        Blockquote(Blockquote),
        OrderedList(OrderedList),
        BulletList(BulletList),
        TodoList(TodoList),
        CodeBlock(CodeBlock),
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Inline {
        Text(Text),
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Text {
        pub text: String,

        pub bold: Option<bool>,

        pub italic: Option<bool>,

        pub underline: Option<bool>,

        pub strikethrough: Option<bool>,

        pub code: Option<bool>,

        pub link: Option<Link>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Link {
        pub href: String,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Heading {
        pub level: HeadingLevel,
        pub content: Vec<Inline>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Serialize_repr, Deserialize_repr)]
    #[repr(u8)]
    pub enum HeadingLevel {
        One = 1,
        Two = 2,
        Three = 3,
        Four = 4,
        Five = 5,
        Six = 6,
    }

    impl From<u8> for HeadingLevel {
        fn from(v: u8) -> Self {
            match v {
                1 => Self::One,
                2 => Self::Two,
                3 => Self::Three,
                4 => Self::Four,
                5 => Self::Five,
                6 => Self::Six,
                _ => Self::One,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Paragraph {
        pub content: Vec<Inline>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Blockquote {
        pub content: Vec<Block>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct OrderedList {
        pub content: Vec<ListItem>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct BulletList {
        pub content: Vec<ListItem>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum ListItem {
        ListItem { content: Vec<Block> },
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct TodoList {
        pub content: Vec<TodoItem>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum TodoItem {
        TodoItem {
            checked: Option<bool>,
            content: Vec<Block>,
        },
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct CodeBlock {
        pub language: Option<String>,

        pub content: Vec<CodeBlockContent>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum CodeBlockContent {
        Text(Text),
    }
}
