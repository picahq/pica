use axum::extract::Query;
use entities::{
    event_access::EventAccess, CONTAINS_FILTER, DELETED_FILTER, DUAL_ENVIRONMENT_HEADER,
    ENVIRONMENT_FILTER, LIMIT_FILTER, OPTIONS_FILTER, OWNERSHIP_FILTER, REGEX_FILTER, SKIP_FILTER,
};
use http::HeaderMap;
use mongodb::bson::{doc, Document};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct MongoQuery {
    pub filter: Document,
    pub skip: u64,
    pub limit: u64,
}

pub fn shape_mongo_filter(
    query: Option<Query<BTreeMap<String, String>>>,
    event_access: Option<Arc<EventAccess>>,
    headers: Option<HeaderMap>,
) -> MongoQuery {
    let mut filter = doc! {};
    let mut skip = 0;
    let mut limit = 20;

    if let Some(q) = query {
        for (key, value) in q.0.iter() {
            if key == LIMIT_FILTER {
                limit = value.parse().unwrap_or(20);
            } else if key == CONTAINS_FILTER {
                let values = string_to_vec(value);
                let splitted = values.split_first();

                let target = splitted.map(|a| a.0);
                let rest = splitted.map(|a| a.1);

                if let (Some(target), Some(rest)) = (target, rest) {
                    filter.insert(target, doc! { CONTAINS_FILTER: rest.to_vec() });
                }
            } else if key == SKIP_FILTER {
                skip = value.parse().unwrap_or(0);
            } else if key == REGEX_FILTER {
                let values = string_to_vec(value);

                let splitted = values.split_first();
                let target = splitted.map(|a| a.0);
                let rest = splitted.and_then(|a| a.1.first());

                if let (Some(target), Some(element)) = (target, rest) {
                    filter.insert(
                        target,
                        doc! { REGEX_FILTER: element.to_string(), OPTIONS_FILTER: "i".to_string() },
                    );
                }
            } else {
                match value.as_str() {
                    "true" => filter.insert(key, true),
                    "false" => filter.insert(key, false),
                    _ => filter.insert(key, value.clone()),
                };
            }
        }
    }

    filter.insert(DELETED_FILTER, false);

    if event_access.is_some() {
        filter.insert(
            OWNERSHIP_FILTER,
            event_access.as_ref().unwrap().ownership.id.to_string(),
        );
        filter.insert(
            ENVIRONMENT_FILTER,
            event_access.as_ref().unwrap().environment.to_string(),
        );

        if let Some(headers) = headers {
            if let Some(show_dual_envs) = headers.get(DUAL_ENVIRONMENT_HEADER) {
                if show_dual_envs == "true" {
                    filter.remove(ENVIRONMENT_FILTER);
                }
            }
        }
    }

    println!("{:#?}", filter);

    MongoQuery {
        filter,
        limit,
        skip,
    }
}

fn string_to_vec(s: &str) -> Vec<String> {
    s.split(',').map(|s| s.to_string()).collect::<Vec<String>>()
}

#[cfg(test)]
mod test {
    use super::shape_mongo_filter;
    use crate::helper::shape_mongo_filter::{
        MongoQuery, DELETED_FILTER, DUAL_ENVIRONMENT_HEADER, ENVIRONMENT_FILTER, LIMIT_FILTER,
        OWNERSHIP_FILTER, SKIP_FILTER,
    };
    use axum::extract::Query;
    use entities::{
        id::{prefix::IdPrefix, Id},
        {
            connection_definition::{ConnectionDefinitionType, Paths},
            environment::Environment,
            event_access::EventAccess,
            ownership::Ownership,
            record_metadata::RecordMetadata,
        },
    };
    use http::HeaderMap;
    use std::{collections::BTreeMap, sync::Arc};

    #[test]
    fn test_shape_mongo_filter() {
        let params = BTreeMap::from([
            (DELETED_FILTER.to_string(), "true".to_string()),
            (OWNERSHIP_FILTER.to_string(), "foo".to_string()),
            (ENVIRONMENT_FILTER.to_string(), "bar".to_string()),
            (SKIP_FILTER.to_string(), "10".to_string()),
            (LIMIT_FILTER.to_string(), "10".to_string()),
        ]);

        let MongoQuery {
            filter: mut doc,
            skip,
            limit,
        } = shape_mongo_filter(Some(Query(params.clone())), None, None);
        assert_eq!(doc.get_str(OWNERSHIP_FILTER).unwrap(), "foo");
        assert_eq!(doc.get_str(ENVIRONMENT_FILTER).unwrap(), "bar");
        assert!(!doc.get_bool(DELETED_FILTER).unwrap());
        assert_eq!(limit, 10);
        assert_eq!(skip, 10);

        doc.insert(DELETED_FILTER, true);
        assert!(doc.get_bool(DELETED_FILTER).unwrap());

        let event_access = Arc::new(EventAccess {
            id: Id::now(IdPrefix::EventAccess),
            name: "name".to_string(),
            key: "key".to_string(),
            namespace: "default".to_string(),
            platform: "stripe".to_string(),
            r#type: ConnectionDefinitionType::Api,
            group: "group".to_string(),
            ownership: Ownership::new("baz".to_string()),
            paths: Paths::default(),
            access_key: "access_key".to_string(),
            environment: Environment::Test,
            record_metadata: RecordMetadata::default(),
            throughput: 1000,
        });

        let MongoQuery { filter: doc, .. } =
            shape_mongo_filter(Some(Query(params)), Some(event_access), None);
        assert_eq!(doc.get_str(OWNERSHIP_FILTER).unwrap(), "baz");
        assert_eq!(doc.get_str(ENVIRONMENT_FILTER).unwrap(), "test");
    }

    #[test]
    fn requesting_dual_environments() {
        let params = BTreeMap::from([
            (DELETED_FILTER.to_string(), "true".to_string()),
            ("ownership.buildableId".to_string(), "foo".to_string()),
            ("environment".to_string(), "bar".to_string()),
        ]);

        let mut headers = HeaderMap::new();
        headers.insert(DUAL_ENVIRONMENT_HEADER, "true".parse().unwrap());

        let event_access = Arc::new(EventAccess {
            id: Id::now(IdPrefix::EventAccess),
            name: "name".to_string(),
            key: "key".to_string(),
            namespace: "default".to_string(),
            platform: "stripe".to_string(),
            r#type: ConnectionDefinitionType::Api,
            group: "group".to_string(),
            ownership: Ownership::new("baz".to_string()),
            paths: Paths::default(),
            access_key: "access_key".to_string(),
            environment: Environment::Test,
            record_metadata: RecordMetadata::default(),
            throughput: 1000,
        });

        let MongoQuery { filter: doc, .. } = shape_mongo_filter(
            Some(Query(params.clone())),
            Some(event_access),
            Some(headers),
        );

        assert!(!doc.contains_key(ENVIRONMENT_FILTER));
    }
}
