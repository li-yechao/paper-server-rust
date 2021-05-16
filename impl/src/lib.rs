pub mod auth;
pub mod paper;
pub mod user;

#[macro_export]
macro_rules! shaku_deref_self_interface {
    ($interface:ident, $target:path) => {
        pub trait $interface: shaku::Interface + std::ops::Deref<Target = $target> {}

        impl std::ops::Deref for $target {
            type Target = Self;

            fn deref(&self) -> &Self::Target {
                self
            }
        }

        impl $interface for $target {}
    };
}

mod utils {
    use bson::*;
    use futures::StreamExt;
    use mongodb::{
        options::{FindOneOptions, FindOptions},
        Collection,
    };
    use paper::{Error, OrderBy, OrderDirection, Pagination, PaginationList, Result};
    use serde::{de::DeserializeOwned, Serialize};

    pub fn new_id() -> String {
        mongodb::bson::oid::ObjectId::new().to_hex()
    }

    fn now() -> std::time::Duration {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Invalid system time")
    }

    pub fn now_msec() -> u64 {
        now().as_millis() as u64
    }

    pub fn to_doc<T: Serialize>(o: &T) -> Result<Document> {
        let mut doc = to_document(o).map_err(|e| Error::unknown(e.to_string()))?;
        if let Some(id) = doc.remove("id") {
            doc.insert("_id", id);
        }
        Ok(doc)
    }

    pub fn from_doc<T: DeserializeOwned>(mut doc: Document) -> Result<T> {
        let id = doc.remove("_id").ok_or_else(|| {
            Error::unknown("Convert document to object require id property exist".to_owned())
        })?;
        doc.insert("id", id);
        from_document(doc).map_err(|e| Error::unknown(e.to_string()))
    }

    pub async fn mongodb_select_pagination<T, K, F, S, O>(
        collection: &Collection,
        pagination: Pagination<K>,
        filter: F,
        order_by: OrderBy<S>,
        list_opts: O,
    ) -> Result<PaginationList<T>>
    where
        T: DeserializeOwned,
        K: ToString,
        F: Into<Option<Document>>,
        S: AsRef<str>,
        O: Into<Option<FindOptions>>,
    {
        let filter: Document = filter.into().unwrap_or(doc! {});

        let (order_direction, order_op) = match (&pagination, order_by.direction) {
            (Pagination::After { .. }, OrderDirection::Asc)
            | (Pagination::Before { .. }, OrderDirection::Desc) => (1 as i32, "$gt"),

            (Pagination::After { .. }, OrderDirection::Desc)
            | (Pagination::Before { .. }, OrderDirection::Asc) => (-1 as i32, "$lt"),
        };
        let order_field = order_by.field.as_ref();

        let (cursor, skip, limit) = match &pagination {
            Pagination::After { after, skip, first } => (after, skip, *first),
            Pagination::Before { before, skip, last } => (before, skip, *last),
        };
        let cursor = cursor.as_ref().map(|x| x.to_string());

        let mut list_opts = list_opts.into().unwrap_or_default();
        list_opts.sort = Some({
            let mut sort = doc! { order_field: order_direction };
            if order_field != "_id" {
                sort.insert("_id", order_direction);
            }
            sort
        });
        list_opts.skip = skip.map(|x| x as i64);
        list_opts.limit = Some(limit as i64 + 1);

        let mut list_filter = match &cursor {
            Some(cursor) => match order_field {
                "_id" => doc! { "_id": { order_op: cursor } },
                order_field => {
                    match collection
                        .find_one(
                            doc! { "_id": cursor },
                            FindOneOptions::builder()
                                .projection(doc! { order_field: 1 })
                                .build(),
                        )
                        .await
                        .map_err(|e| Error::unknown(e.to_string()))?
                        .as_ref()
                        .and_then(|doc| doc.get(order_field))
                    {
                        Some(order_field_value) => doc! {
                            "$or": [
                                { order_field: { order_op: order_field_value } },
                                {
                                    order_field: order_field_value,
                                    "_id": { order_op: cursor },
                                }
                            ],
                        },
                        None => doc! { "_id": { order_op: cursor } },
                    }
                }
            },
            None => doc! {},
        };

        list_filter.extend(filter.clone());

        let mut list = collection
            .find(list_filter, list_opts)
            .await
            .map_err(|e| Error::unknown(e.to_string()))?
            .map(|x| {
                x.map_err(|e| Error::unknown(e.to_string()))
                    .and_then(|x| from_doc::<T>(x))
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<T>>>()?;

        if let Pagination::Before { .. } = pagination {
            list.reverse();
        }

        let total = collection
            .count_documents(filter, None)
            .await
            .map_err(|e| Error::unknown(e.to_string()))? as u64;

        let has_next_page = list.len() > limit as usize;
        if has_next_page {
            list.pop();
        }

        Ok(PaginationList {
            list,
            total,
            has_next_page,
        })
    }
}
