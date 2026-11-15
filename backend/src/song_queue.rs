use std::collections::VecDeque;
use std::mem::take;
use std::time::Duration;

use futures::StreamExt;
use log::error;
use smol::{Executor, Task, Timer, channel::Sender, lock::Mutex};

use crate::{
    AppState, HandlerEvent,
    yt_dlp::{YoutubeInfo, YtdlpResult, get_ytdlp},
};

#[derive(Debug, Default)]
pub struct SongQueue<'ex> {
    id_counter: u16,
    queue: VecDeque<(u16, QueueEntry)>,
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
    const PERIOD: Duration = Duration::from_millis(100);
    let mut timer = Timer::interval(PERIOD);

    let queue_push = |id, state: &mut AppState, entry| {
        state.queue.queue.push_back((id, entry));
    };

    let queue_push_new = |state: &mut AppState, entry| {
        let id = state.queue.new_id();
        queue_push(id, state, entry);
    };

    loop {
        let mut queue_changed = false;
        {
            let mut state = state.lock().await;
            if state.queue.executor.try_tick() {
                let old_queue = take(&mut state.queue.queue);
                let mut fetching_counter = 0;
                for (id, entry) in old_queue {
                    match entry {
                        QueueEntry::Fetched(info) => {
                            queue_push(id, &mut state, QueueEntry::Fetched(info));
                        }
                        QueueEntry::Fetching(task) => {
                            if task.task.is_finished() {
                                queue_changed = true;
                                match task.task.await {
                                    Ok(YtdlpResult::Single(info)) => {
                                        queue_push(id, &mut state, QueueEntry::Fetched(info));
                                    }
                                    Ok(YtdlpResult::Playlist(list)) => {
                                        for info in list {
                                            let url = format!(
                                                "https://www.youtube.com/watch?v={}",
                                                info.id
                                            );
                                            let title = info.title;
                                            if fetching_counter < 5 {
                                                let future = get_ytdlp(url.clone());
                                                let task = state.queue.executor.spawn(future);
                                                let task = RefetchTask { url, title, task };
                                                queue_push_new(
                                                    &mut state,
                                                    QueueEntry::Refetching(task),
                                                );
                                                fetching_counter += 1;
                                            } else {
                                                let task = PendingRefetchTask { url, title };
                                                queue_push_new(
                                                    &mut state,
                                                    QueueEntry::PendingRefetch(task),
                                                );
                                            }
                                        }
                                    }
                                    Err(error) => {
                                        error!("yt-dlp Failed: {error}");
                                    }
                                };
                            } else {
                                queue_push(id, &mut state, QueueEntry::Fetching(task));
                                fetching_counter += 1;
                            }
                        }
                        QueueEntry::Refetching(task) => {
                            if task.task.is_finished() {
                                queue_changed = true;
                                match task.task.await {
                                    Ok(YtdlpResult::Single(info)) => {
                                        queue_push(id, &mut state, QueueEntry::Fetched(info));
                                    }
                                    Err(error) => {
                                        error!("yt-dlp Failed: {error}");
                                    }
                                    Ok(YtdlpResult::Playlist(_)) => {
                                        unreachable!();
                                    }
                                };
                            } else {
                                queue_push(id, &mut state, QueueEntry::Refetching(task));
                                fetching_counter += 1;
                            }
                        }
                        QueueEntry::PendingRefetch(task) => {
                            if fetching_counter < 5 {
                                let future = get_ytdlp(task.url.clone());
                                let task = RefetchTask {
                                    url: task.url,
                                    title: task.title,
                                    task: state.queue.executor.spawn(future),
                                };
                                queue_push(id, &mut state, QueueEntry::Refetching(task));
                                fetching_counter += 1;
                            } else {
                                queue_push(id, &mut state, QueueEntry::PendingRefetch(task));
                            }
                        }
                    }
                }
            }
        }
        if queue_changed {
            let _ = handler_event_tx.send(HandlerEvent::UpdateQueue).await;
        }
        timer.next().await;
    }
}

impl<'ex> SongQueue<'ex> {
    fn new_id(&mut self) -> u16 {
        let id = self.id_counter;
        self.id_counter += 1;
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

    pub async fn try_pop(&mut self) -> Option<Option<(u16, YoutubeInfo)>> {
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

    pub fn try_delete(&mut self, id: u16) {
        self.queue.retain(|(i, _)| *i != id);
    }

    pub fn iter(&self) -> impl Iterator<Item = (u16, &QueueEntry)> {
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
