use crate::api::v1::gen::PageToken;

pub trait Paginator: Sized {
    fn limit(&self) -> i32;
    fn offset(&self) -> i32;
    fn next_page<T>(&self, data: &mut Vec<T>) -> Option<Self>;
}

impl Paginator for PageToken {
    /// 多取一条数据
    fn limit(&self) -> i32 {
        self.limit + 1
    }

    fn offset(&self) -> i32 {
        self.offset
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
