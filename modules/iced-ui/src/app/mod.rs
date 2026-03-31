use crate::app::root::{RootMsg, RootState};
use crate::utils::ResultExt;
use avisaver_core::{APPLICATION_ID, APPLICATION_NAME, APPLICATION_TITLE};
use avisaver_osc::error::OSCStartupError;
use avisaver_osc::{OSCListener, OSCQuery, QueryOptions};
use enumset::{EnumSet, EnumSetType};
use iced::widget::{column, container, progress_bar, space, text};
use iced::window::Position;
use iced::window::settings::PlatformSpecific;
use iced::{Element, Size, Subscription, Task, Theme, window};
use rosc::OscPacket;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::ReceiverStream;

pub mod icons;
pub mod root;
pub mod settings;
pub mod styles;

const APPLICATION_SPLASH_TITLE: &str = "AviSaver Starting...";

#[derive(Clone)]
pub enum ASMsg {
    OSCQueryStarted(Arc<Mutex<Option<Result<OSCQuery, OSCStartupError>>>>),
    OSCPacket(SocketAddr, OscPacket),
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    SplashClose(window::Id),
    StartupTaskFinished(StartupTask),
    ShutdownTaskFinished(ShutdownTask),
    Root(RootMsg),
}

pub struct ASState {
    splash_window: Option<window::Id>,
    root_window: Option<window::Id>,
    running_startup_tasks: EnumSet<StartupTask>,
    running_shutdown_tasks: EnumSet<ShutdownTask>,

    osc: Option<OSCQuery>,

    root: RootState,
}

#[derive(EnumSetType, Debug)]
pub enum StartupTask {
    LoadSettings,
    StartOSCQuery,
}

#[derive(EnumSetType, Debug)]
pub enum ShutdownTask {
    OSCShutdown,
}

impl ASState {
    pub fn new() -> (Self, Task<ASMsg>) {
        info!("Launching application...");

        let (splash_id, window_task) = window::open(window::Settings {
            size: Size::new(480.0, 270.0),
            position: Position::Centered,
            icon: None,
            platform_specific: PlatformSpecific {
                application_id: format!("{APPLICATION_ID}.splash"),
                ..Default::default()
            },
            decorations: false,
            resizable: false,
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

        let load_settings = Task::future(async {
            avisaver_core::settings::init().await;
            ASMsg::StartupTaskFinished(StartupTask::LoadSettings)
        });

        (
            Self {
                splash_window: Some(splash_id),
                root_window: None,
                running_startup_tasks: EnumSet::all(),
                running_shutdown_tasks: Default::default(),
                osc: None,
                root: RootState::new(),
            },
            Task::batch([window_task.discard(), osc_events, start_osc, load_settings]),
        )
    }

    pub fn update(&mut self, msg: ASMsg) -> Task<ASMsg> {
        match msg {
            ASMsg::WindowClosed(id) => {
                if self.splash_window.is_some_and(|splash_id| splash_id == id)
                    || self.root_window.is_some_and(|root_id| root_id == id)
                {
                    self.start_shutdown()
                } else {
                    // TODO: handle other window closes
                    Task::none()
                }
            }
            ASMsg::WindowOpened(id) => {
                if self.root_window.is_some_and(|root| root == id) {
                    if let Some(splash_id) = self.splash_window {
                        Task::future(async move {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            ASMsg::SplashClose(splash_id)
                        })
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            ASMsg::SplashClose(id) => {
                self.splash_window = None;
                window::close(id)
            }
            ASMsg::StartupTaskFinished(task) => self.check_startup(task),
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
                    self.check_startup(StartupTask::StartOSCQuery)
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

    fn check_startup(&mut self, completed: StartupTask) -> Task<ASMsg> {
        info!("Task {completed:?} completed");

        self.running_startup_tasks.remove(completed);
        if self.running_startup_tasks.is_empty() {
            self.do_startup()
        } else {
            Task::none()
        }
    }

    fn do_startup(&mut self) -> Task<ASMsg> {
        info!("All tasks completed, launching main window...");

        let (id, window_task) = window::open(window::Settings {
            size: Size::new(1280.0, 720.0),
            position: Position::Centered,
            icon: None,
            platform_specific: PlatformSpecific {
                application_id: APPLICATION_ID.to_string(),
                ..Default::default()
            },
            ..Default::default()
        });

        self.root_window = Some(id);

        window_task.discard()
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
        let task_count = EnumSet::<StartupTask>::variant_count();

        if self
            .splash_window
            .is_some_and(|splash_window| splash_window == window_id)
        {
            column![
                container(text(APPLICATION_TITLE).size(48)).padding(10),
                space::vertical(),
                container(text("Loading...")).padding(10),
                progress_bar(
                    0.0..=(task_count as f32),
                    (task_count - self.running_startup_tasks.len() as u32) as f32
                ),
            ]
            .into()
        } else if self
            .root_window
            .is_some_and(|root_window| root_window == window_id)
        {
            self.root.view().map(ASMsg::Root)
        } else {
            space().into()
        }
    }

    pub fn subscriptions(&self) -> Subscription<ASMsg> {
        Subscription::batch([
            window::close_events().map(ASMsg::WindowClosed),
            window::open_events().map(ASMsg::WindowOpened),
        ])
    }

    pub fn theme(&self, _window_id: window::Id) -> Theme {
        Theme::TokyoNight
    }

    pub fn title(&self, window_id: window::Id) -> String {
        if self
            .splash_window
            .is_some_and(|splash_window| splash_window == window_id)
        {
            APPLICATION_SPLASH_TITLE.to_string()
        } else if self
            .root_window
            .is_some_and(|root_window| root_window == window_id)
        {
            APPLICATION_TITLE.to_string()
        } else {
            "".to_string()
        }
    }
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
