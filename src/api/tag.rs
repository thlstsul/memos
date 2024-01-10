use super::{
    v2::{ListTagsResponse, Tag},
    USER_NAME_PREFIX,
};

impl From<Vec<Tag>> for ListTagsResponse {
    fn from(tags: Vec<Tag>) -> Self {
        let mut tags = tags;
        tags.iter_mut()
            .for_each(|i| i.creator = format!("{}/{}", USER_NAME_PREFIX, i.creator));

        Self { tags }
    }
}
