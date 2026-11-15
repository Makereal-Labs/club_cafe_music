use std::collections::VecDeque;
use std::mem::take;
use std::time::Duration;

use futures::StreamExt;
use log::error;
use smol::{Executor, Task, Timer, channel::Sender, lock::Mutex};

use crate::AppState;
use crate::HandlerEvent;
use crate::yt_dlp::{YoutubeInfo, YtdlpResult, get_ytdlp};

const PROCESS_PERIOD: Duration = Duration::from_millis(100);
const MAX_FETCH_COUNT: u8 = 5;

pub type QueueEntryId = u16;

#[derive(Debug, Default)]
pub struct SongQueue<'ex> {
    id_counter: QueueEntryId,
    queue: VecDeque<(QueueEntryId, QueueEntry)>,
    executor: Executor<'ex>,
}

#[derive(Debug)]
pub enum QueueEntry {
    Fetched(YoutubeInfo),
    Fetching(FetchTask),
    Refetching(RefetchTask),
    PendingRefetch(PendingRefetchTask),
}

#[derive(Debug)]
pub struct FetchTask {
    url: String,
    task: Task<anyhow::Result<YtdlpResult>>,
}

#[derive(Debug)]
pub struct RefetchTask {
    url: String,
    title: String,
    task: Task<anyhow::Result<YtdlpResult>>,
}

#[derive(Debug)]
pub struct PendingRefetchTask {
    url: String,
    title: String,
}

pub async fn process_queue(
    state: &Mutex<AppState<'_>>,
    handler_event_tx: Sender<HandlerEvent>,
) -> ! {
    let mut timer = Timer::interval(PROCESS_PERIOD);

    loop {
        let queue_changed = {
            let mut state = state.lock().await;
            if state.queue.executor.try_tick() {
                tick_queue(&mut state.queue).await
            } else {
                false
            }
        };
        if queue_changed {
            let _ = handler_event_tx.send(HandlerEvent::UpdateQueue).await;
        }
        timer.next().await;
    }
}

async fn tick_queue(queue: &mut SongQueue<'_>) -> bool {
    let mut queue_changed = false;
    let mut fetching_counter = 0;
    let old_queue = take(&mut queue.queue);

    for (id, entry) in old_queue {
        match entry {
            QueueEntry::Fetched(info) => {
                let entry = QueueEntry::Fetched(info);
                queue.queue.push_back((id, entry));
            }
            QueueEntry::Fetching(task) => {
                if task.task.is_finished() {
                    queue_changed = true;
                    let task_result = task.task.await;
                    process_task_result(task_result, id, queue, &mut fetching_counter);
                } else {
                    let entry = QueueEntry::Fetching(task);
                    queue.queue.push_back((id, entry));
                    fetching_counter += 1;
                }
            }
            QueueEntry::Refetching(task) => {
                if task.task.is_finished() {
                    queue_changed = true;
                    let task_result = task.task.await;
                    process_task_result(task_result, id, queue, &mut fetching_counter);
                } else {
                    let entry = QueueEntry::Refetching(task);
                    queue.queue.push_back((id, entry));
                    fetching_counter += 1;
                }
            }
            QueueEntry::PendingRefetch(task) => {
                if fetching_counter < MAX_FETCH_COUNT {
                    let future = get_ytdlp(task.url.clone());
                    let task = RefetchTask {
                        url: task.url,
                        title: task.title,
                        task: queue.executor.spawn(future),
                    };
                    let entry = QueueEntry::Refetching(task);
                    queue.queue.push_back((id, entry));
                    fetching_counter += 1;
                } else {
                    let entry = QueueEntry::PendingRefetch(task);
                    queue.queue.push_back((id, entry));
                }
            }
        }
    }
    queue_changed
}

fn process_task_result(
    task_result: Result<YtdlpResult, anyhow::Error>,
    id: u16,
    queue: &mut SongQueue<'_>,
    fetching_counter: &mut u8,
) {
    match task_result {
        Ok(YtdlpResult::Single(info)) => {
            let entry = QueueEntry::Fetched(info);
            queue.queue.push_back((id, entry));
        }
        Ok(YtdlpResult::Playlist(list)) => {
            for info in list {
                let url = format!("https://www.youtube.com/watch?v={}", info.id);
                let title = info.title;
                if *fetching_counter < MAX_FETCH_COUNT {
                    let future = get_ytdlp(url.clone());
                    let task = queue.executor.spawn(future);
                    let task = RefetchTask { url, title, task };
                    let id = queue.new_id();
                    let entry = QueueEntry::Refetching(task);
                    queue.queue.push_back((id, entry));
                    *fetching_counter += 1;
                } else {
                    let task = PendingRefetchTask { url, title };
                    let id = queue.new_id();
                    let entry = QueueEntry::PendingRefetch(task);
                    queue.queue.push_back((id, entry));
                }
            }
        }
        Err(error) => {
            error!("yt-dlp Failed: {error}");
        }
    }
}

impl<'ex> SongQueue<'ex> {
    fn new_id(&mut self) -> QueueEntryId {
        let id = self.id_counter;
        self.id_counter = self.id_counter.wrapping_add(1);
        id
    }

    pub fn push_task(
        &mut self,
        future: impl Future<Output = anyhow::Result<YtdlpResult>> + Send + 'ex,
        url: String,
    ) {
        let task = self.executor.spawn(future);
        let task = FetchTask { task, url };
        let id = self.new_id();
        self.queue.push_back((id, QueueEntry::Fetching(task)));
    }

    pub async fn try_pop(&mut self) -> Option<Option<(QueueEntryId, YoutubeInfo)>> {
        let Some((id, first)) = self.queue.pop_front() else {
            return Some(None);
        };
        match first {
            QueueEntry::Fetched(info) => Some(Some((id, info))),
            QueueEntry::Fetching(task) => {
                self.queue.push_front((id, QueueEntry::Fetching(task)));
                None
            }
            QueueEntry::Refetching(task) => {
                self.queue.push_front((id, QueueEntry::Refetching(task)));
                None
            }
            QueueEntry::PendingRefetch(task) => {
                self.queue
                    .push_front((id, QueueEntry::PendingRefetch(task)));
                None
            }
        }
    }

    pub fn try_delete(&mut self, id: QueueEntryId) {
        self.queue.retain(|(i, _)| *i != id);
    }

    pub fn iter(&self) -> impl Iterator<Item = (QueueEntryId, &QueueEntry)> {
        self.queue.iter().map(|(a, b)| (*a, b))
    }
}

impl FetchTask {
    pub fn url(&self) -> &str {
        &self.url
    }
}

impl RefetchTask {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}

impl PendingRefetchTask {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}
