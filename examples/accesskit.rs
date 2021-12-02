use accesskit_schema::{Node, NodeId, Role, StringEncoding, Tree, TreeId, TreeUpdate};
use simple_logger::SimpleLogger;
use std::{
    num::NonZeroU64,
    sync::{Arc, Mutex},
};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{AccessKitFactory, Window, WindowBuilder, WindowId},
};

const WINDOW_TITLE: &str = "Hello world";

const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(1) });
const BUTTON_1_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(2) });
const BUTTON_2_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(3) });
const INITIAL_FOCUS: NodeId = BUTTON_1_ID;

fn make_button(id: NodeId, name: &str) -> Node {
    Node {
        name: Some(name.into()),
        focusable: true,
        ..Node::new(id, Role::Button)
    }
}

#[derive(Debug)]
struct State {
    focus: NodeId,
    is_window_focused: bool,
}

impl State {
    fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            focus: INITIAL_FOCUS,
            is_window_focused: false,
        }))
    }

    fn update_focus(&mut self, window: &Window, is_window_focused: bool) {
        self.is_window_focused = is_window_focused;
        window.update_accesskit_if_active(|| TreeUpdate {
            clear: None,
            nodes: vec![],
            tree: None,
            focus: is_window_focused.then(|| self.focus),
        });
    }
}

#[derive(Debug)]
struct MyAccessKitFactory(Arc<Mutex<State>>);

impl AccessKitFactory for MyAccessKitFactory {
    fn initial_tree_for_window(&self, _id: WindowId) -> TreeUpdate {
        let state = self.0.lock().unwrap();
        let root = Node {
            children: Box::new([BUTTON_1_ID, BUTTON_2_ID]),
            name: Some(WINDOW_TITLE.into()),
            ..Node::new(WINDOW_ID, Role::Window)
        };
        let button_1 = make_button(BUTTON_1_ID, "Button 1");
        let button_2 = make_button(BUTTON_2_ID, "Button 2");
        TreeUpdate {
            clear: None,
            nodes: vec![root, button_1, button_2],
            tree: Some(Tree::new(
                TreeId("test".into()),
                WINDOW_ID,
                StringEncoding::Utf8,
            )),
            focus: state.is_window_focused.then(|| state.focus),
        }
    }
}

fn main() {
    SimpleLogger::new().init().unwrap();
    let event_loop = EventLoop::new();

    let state = State::new();
    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_accesskit_factory(MyAccessKitFactory(Arc::clone(&state)))
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Focused(is_window_focused) => {
                        let mut state = state.lock().unwrap();
                        state.update_focus(&window, is_window_focused);
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(virtual_code),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        match virtual_code {
                            VirtualKeyCode::Tab => {
                                let mut state = state.lock().unwrap();
                                state.focus = if state.focus == BUTTON_1_ID {
                                    BUTTON_2_ID
                                } else {
                                    BUTTON_1_ID
                                };
                                state.update_focus(&window, true);
                            }
                            VirtualKeyCode::Space => {
                                // This is a pretty hacky way of updating a node.
                                // A real GUI framework would have a consistent
                                // way of building a node from underlying data.
                                let focus = state.lock().unwrap().focus;
                                let node = if focus == BUTTON_1_ID {
                                    make_button(BUTTON_1_ID, "You pressed button 1")
                                } else {
                                    make_button(BUTTON_2_ID, "You pressed button 2")
                                };
                                let update = TreeUpdate {
                                    clear: None,
                                    nodes: vec![node],
                                    tree: None,
                                    focus: Some(focus),
                                };
                                window.update_accesskit(update);
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    });
}
