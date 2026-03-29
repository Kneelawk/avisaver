use crate::app::root::{RootMsg, RootState};
use crate::utils::ResultExt;
use avisaver_osc::error::OSCStartupError;
use avisaver_osc::{OSCListener, OSCQuery, QueryOptions};
use enumset::{EnumSet, EnumSetType};
use iced::widget::{column};
use iced::window::Position;
use iced::window::settings::PlatformSpecific;
use iced::{Element, Size, Subscription, Task, Theme, window};
use rosc::OscPacket;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::ReceiverStream;

pub mod root;
pub mod styles;
pub mod icons;

const APPLICATION_ID: &str = "com.kneelawk.avisaver";
const APPLICATION_NAME: &str = "avisaver";
const APPLICATION_TITLE: &str = "AviSaver";

#[derive(Clone)]
pub enum ASMsg {
    OSCQueryStarted(Arc<Mutex<Option<Result<OSCQuery, OSCStartupError>>>>),
    OSCPacket(SocketAddr, OscPacket),
    WindowClosed(window::Id),
    ShutdownTaskFinished(ShutdownTask),
    Root(RootMsg),
}

pub struct ASState {
    root_window: window::Id,
    running_shutdown_tasks: EnumSet<ShutdownTask>,

    osc: Option<OSCQuery>,

    root: RootState,
}

impl ASState {
    pub fn new() -> (Self, Task<ASMsg>) {
        info!("Launching application...");

        let (id, task) = window::open(window::Settings {
            size: Size::new(1280.0, 720.0),
            position: Position::Centered,
            icon: None,
            platform_specific: PlatformSpecific {
                application_id: APPLICATION_ID.to_string(),
                ..Default::default()
            },
            ..Default::default()
        });

        let (osc_tx, osc_rx) = mpsc::channel(64);
        let osc_events = Task::stream(ReceiverStream::new(osc_rx));
        let start_osc = Task::future(async {
            ASMsg::OSCQueryStarted(Arc::new(Mutex::new(Some(
                OSCQuery::new(QueryOptions {
                    app_name: APPLICATION_NAME.to_string(),
                    directories: vec!["/avatar".to_string()],
                    listener: ASOSCListener { tx: osc_tx },
                })
                .await,
            ))))
        });

        (
            Self {
                root_window: id,
                running_shutdown_tasks: Default::default(),
                osc: None,
                root: RootState::new(),
            },
            Task::batch([task.discard(), osc_events, start_osc]),
        )
    }

    pub fn update(&mut self, msg: ASMsg) -> Task<ASMsg> {
        match msg {
            ASMsg::WindowClosed(id) => {
                if id == self.root_window {
                    self.start_shutdown()
                } else {
                    // TODO: handle other window closes
                    Task::none()
                }
            }
            ASMsg::ShutdownTaskFinished(task) => {
                self.running_shutdown_tasks.remove(task);
                if self.running_shutdown_tasks.is_empty() {
                    self.finish_shutdown()
                } else {
                    Task::none()
                }
            }
            ASMsg::OSCQueryStarted(res) => match res
                .try_lock()
                .expect("OSCQuery mutex already borrowed??? something very bad has happened")
                .take()
                .expect("OSCQuery already taken??? something very bad has happened")
            {
                Ok(osc) => {
                    self.osc = Some(osc);
                    Task::none()
                }
                Err(err) => {
                    error!(
                        "Error starting OSCQuery. AviSaver cannot run without OSCQuery. Error: {err:?}"
                    );
                    self.start_shutdown()
                }
            },
            ASMsg::Root(msg) => self.root.update(msg).map(ASMsg::Root),
            _ => Task::none(),
        }
    }

    fn start_shutdown(&mut self) -> Task<ASMsg> {
        info!("Shutting down AviSaver...");

        let mut tasks = vec![];

        if let Some(mut osc) = self.osc.take() {
            self.running_shutdown_tasks
                .insert(ShutdownTask::OSCShutdown);
            tasks.push(Task::future(async move {
                osc.shutdown().await.error("Error shutting down OSC server");
                ASMsg::ShutdownTaskFinished(ShutdownTask::OSCShutdown)
            }));
        }

        if self.running_shutdown_tasks.is_empty() {
            return self.finish_shutdown();
        }

        Task::batch(tasks)
    }

    fn finish_shutdown(&self) -> Task<ASMsg> {
        info!("Cleanup done. Goodbye! ^-^");

        iced::exit()
    }

    pub fn view(&'_ self, window_id: window::Id) -> Element<'_, ASMsg> {
        if window_id == self.root_window {
            self.root.view().map(ASMsg::Root)
        } else {
            column([]).into()
        }
    }

    pub fn subscriptions(&self) -> Subscription<ASMsg> {
        window::close_events().map(ASMsg::WindowClosed)
    }

    pub fn theme(&self, _window_id: window::Id) -> Theme {
        Theme::TokyoNight
    }

    pub fn title(&self, window_id: window::Id) -> String {
        if window_id == self.root_window {
            APPLICATION_TITLE.to_string()
        } else {
            "".to_string()
        }
    }
}

#[derive(EnumSetType, Debug)]
pub enum ShutdownTask {
    OSCShutdown,
}

#[derive(Debug, Clone)]
pub enum MenuButton {
    File,
}

struct ASOSCListener {
    tx: mpsc::Sender<ASMsg>,
}

#[allow(refining_impl_trait_internal)]
impl OSCListener for ASOSCListener {
    async fn packet_received(&self, from: SocketAddr, packet: OscPacket) {
        self.tx
            .send(ASMsg::OSCPacket(from, packet))
            .await
            .warn("Error sending OSC packet");
    }
}
