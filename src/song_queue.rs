use std::collections::VecDeque;

use smol::Task;

use crate::yt_dlp::YoutubeInfo;

#[derive(Debug, Default)]
pub struct SongQueue {
    queue: VecDeque<QueueEntry>,
}

#[derive(Debug)]
pub enum QueueEntry {
    Fetched(YoutubeInfo),
    Fetching(FetchTask),
}

#[derive(Debug)]
pub struct FetchTask {
    url: String,
    task: Task<anyhow::Result<Vec<YoutubeInfo>>>,
}

impl SongQueue {
    pub fn push(&mut self, item: QueueEntry) {
        self.queue.push_back(item);
    }

    pub async fn wait_pop(&mut self) -> Option<YoutubeInfo> {
        let first = self.queue.pop_front()?;
        match first {
            QueueEntry::Fetched(info) => Some(info),
            QueueEntry::Fetching(task) => {
                let result = task.task.await.unwrap();
                let mut list = VecDeque::from(result);
                let info = list.pop_front();
                while let Some(info) = list.pop_back() {
                    self.queue.push_front(QueueEntry::Fetched(info));
                }
                info
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &QueueEntry> {
        self.queue.iter()
    }
}

impl FetchTask {
    pub fn new(task: Task<anyhow::Result<Vec<YoutubeInfo>>>, url: String) -> Self {
        FetchTask { task, url }
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}
