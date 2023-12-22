use super::{
    get_name_parent_token,
    v2::{ListTagsRequest, ListTagsResponse, Tag},
    Error, USER_NAME_PREFIX,
};

impl ListTagsRequest {
    pub fn get_creator(&self) -> Result<String, Error> {
        get_name_parent_token(self.user.clone(), USER_NAME_PREFIX)
    }
}

impl From<Vec<Tag>> for ListTagsResponse {
    fn from(tags: Vec<Tag>) -> Self {
        let mut tags = tags;
        tags.iter_mut()
            .for_each(|i| i.creator = format!("{}/{}", USER_NAME_PREFIX, i.creator));

        Self { tags }
    }
}
