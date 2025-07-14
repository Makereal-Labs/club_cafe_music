use std::{collections::VecDeque, mem::take, time::Duration};

use futures::StreamExt;
use log::error;
use smol::{Executor, Task, Timer, channel::Sender, lock::Mutex};

use crate::{
    AppState, HandlerEvent,
    yt_dlp::{YoutubeInfo, YtdlpResult, get_ytdlp},
};

#[derive(Debug, Default)]
pub struct SongQueue<'ex> {
    queue: VecDeque<QueueEntry>,
    executor: Executor<'ex>,
}

#[derive(Debug)]
pub enum QueueEntry {
    Fetched(YoutubeInfo),
    Fetching(FetchTask),
    Refetching(RefetchTask),
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

pub async fn process_queue(state: &Mutex<AppState<'_>>, handler_event_tx: Sender<HandlerEvent>) {
    const PERIOD: Duration = Duration::from_millis(100);
    let mut timer = Timer::interval(PERIOD);
    loop {
        let mut queue_changed = false;
        {
            let mut state = state.lock().await;
            if state.queue.executor.try_tick() {
                let old_queue = take(&mut state.queue.queue);
                for entry in old_queue {
                    match entry {
                        QueueEntry::Fetched(info) => {
                            state.queue.queue.push_back(QueueEntry::Fetched(info));
                        }
                        QueueEntry::Fetching(task) => {
                            if task.task.is_finished() {
                                queue_changed = true;
                                match task.task.await {
                                    Ok(YtdlpResult::Single(info)) => {
                                        state.queue.queue.push_back(QueueEntry::Fetched(info));
                                    }
                                    Ok(YtdlpResult::Playlist(list)) => {
                                        for info in list {
                                            let url = format!(
                                                "https://www.youtube.com/watch?v={}",
                                                info.id
                                            );
                                            let title = info.title;
                                            let future = get_ytdlp(url.clone());
                                            let task = state.queue.executor.spawn(future);
                                            let task = RefetchTask { url, title, task };
                                            state
                                                .queue
                                                .queue
                                                .push_back(QueueEntry::Refetching(task));
                                        }
                                    }
                                    Err(error) => {
                                        error!("yt-dlp Failed: {error}");
                                    }
                                };
                            } else {
                                state.queue.queue.push_back(QueueEntry::Fetching(task));
                            }
                        }
                        QueueEntry::Refetching(task) => {
                            if task.task.is_finished() {
                                queue_changed = true;
                                match task.task.await {
                                    Ok(YtdlpResult::Single(info)) => {
                                        state.queue.queue.push_back(QueueEntry::Fetched(info));
                                    }
                                    Err(error) => {
                                        error!("yt-dlp Failed: {error}");
                                    }
                                    Ok(YtdlpResult::Playlist(_)) => {
                                        unreachable!();
                                    }
                                };
                            } else {
                                state.queue.queue.push_back(QueueEntry::Refetching(task));
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
    pub fn push_task(
        &mut self,
        future: impl Future<Output = anyhow::Result<YtdlpResult>> + Send + 'ex,
        url: String,
    ) {
        let task = self.executor.spawn(future);
        let task = FetchTask { task, url };
        self.queue.push_back(QueueEntry::Fetching(task));
    }

    pub async fn try_pop(&mut self) -> Option<Option<YoutubeInfo>> {
        let Some(first) = self.queue.pop_front() else {
            return Some(None);
        };
        match first {
            QueueEntry::Fetched(info) => Some(Some(info)),
            QueueEntry::Fetching(task) => {
                self.queue.push_front(QueueEntry::Fetching(task));
                None
            }
            QueueEntry::Refetching(task) => {
                self.queue.push_front(QueueEntry::Refetching(task));
                None
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &QueueEntry> {
        self.queue.iter()
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
