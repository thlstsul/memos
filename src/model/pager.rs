use crate::api::v1::gen::PageToken;

pub trait Paginator: Sized {
    fn as_limit_sql(&self) -> String;
    fn next_page<T>(&self, data: &mut Vec<T>) -> Option<Self>;
}

impl Paginator for PageToken {
    /// 多取一条数据
    fn as_limit_sql(&self) -> String {
        format!("LIMIT {} OFFSET {}", self.limit + 1, self.offset)
    }

    fn next_page<T>(&self, data: &mut Vec<T>) -> Option<Self> {
        if data.len() > self.limit as usize {
            data.pop();
            Some(Self {
                limit: self.limit,
                offset: self.offset + self.limit,
            })
        } else {
            None
        }
    }
}
