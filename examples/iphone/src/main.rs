use std::sync::RwLock;

use cacao::uikit::{
    App, AppDelegate, Scene, SceneConfig, SceneConnectionOptions, SceneSession, Window,
    WindowSceneDelegate,
};

use cacao::color::Color;
use cacao::layout::{Layout, LayoutConstraint};
use cacao::view::{View, ViewController, ViewDelegate};

#[derive(Default)]
struct TestApp;

impl AppDelegate for TestApp {
    fn config_for_scene_session(
        &self,
        session: SceneSession,
        _options: SceneConnectionOptions,
    ) -> SceneConfig {
        SceneConfig::new("Default Configuration", session.role())
    }
}
pub struct RootView {
    pub green: View,
    pub blue: View,
}

impl Default for RootView {
    fn default() -> Self {
        RootView {
            green: View::new(),
            blue: View::new(),
        }
    }
}

impl ViewDelegate for RootView {
    const NAME: &'static str = "RootView";

    fn did_load(&mut self, view: View) {
        self.green.set_background_color(Color::SystemGreen);
        view.add_subview(&self.green);

        self.blue.set_background_color(Color::SystemBlue);
        view.add_subview(&self.blue);

        LayoutConstraint::activate(&[
            self.green
                .leading
                .constraint_equal_to(&view.leading)
                .offset(16.),
            self.green
                .trailing
                .constraint_equal_to(&view.trailing)
                .offset(-16.),
            self.green.height.constraint_equal_to_constant(120.),
            self.blue
                .top
                .constraint_equal_to(&self.green.bottom)
                .offset(16.),
            self.blue
                .leading
                .constraint_equal_to(&view.leading)
                .offset(16.),
            self.blue
                .trailing
                .constraint_equal_to(&view.trailing)
                .offset(-16.),
            self.blue
                .bottom
                .constraint_equal_to(&view.bottom)
                .offset(-16.),
        ]);
    }
}

#[derive(Default)]
pub struct WindowScene {
    pub window: RwLock<Option<Window>>,
    pub root_view_controller: RwLock<Option<ViewController<RootView>>>,
}

impl WindowSceneDelegate for WindowScene {
    fn will_connect(&self, scene: Scene, session: SceneSession, options: SceneConnectionOptions) {
        let bounds = scene.get_bounds();
        let mut window = Window::new(bounds);
        window.set_window_scene(scene);

        let root_view_controller = ViewController::new(RootView::default());
        window.set_root_view_controller(&root_view_controller);
        window.show();

        {
            let mut w = self.window.write().unwrap();
            *w = Some(window);

            let mut vc = self.root_view_controller.write().unwrap();
            *vc = Some(root_view_controller);
        }
    }
}

fn main() {
    App::new(TestApp::default(), || Box::new(WindowScene::default())).run();
}
