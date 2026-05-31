use hudhudscript_ui_core::components::text::TextWidget;
use hudhudscript_ui_core::{
    app::{TuiApp, WidgetTree},
    event::{Event, EventResult, KeyCode, KeyEvent},
    widget::{Rect, RenderCommand, TextStyle},
};

#[test]
fn test_widget_tree_new() {
    let tree = WidgetTree::new("root");
    assert_eq!(tree.id, "root");
    assert!(tree.commands.is_empty());
    assert!(tree.children.is_empty());
}

#[test]
fn test_widget_tree_leaf() {
    let tw = TextWidget::new("Hello");
    let tree = WidgetTree::leaf("text1", &tw, Rect::new(0, 0, 80, 1));
    assert_eq!(tree.id, "text1");
    assert_eq!(tree.commands.len(), 1);
}

#[test]
fn test_widget_tree_composition() {
    let tw1 = TextWidget::new("Title");
    let tw2 = TextWidget::new("Body");
    let tree = WidgetTree::new("root")
        .add_child(WidgetTree::leaf("title", &tw1, Rect::new(0, 0, 80, 1)))
        .add_child(WidgetTree::leaf("body", &tw2, Rect::new(0, 1, 80, 23)));
    assert_eq!(tree.node_count(), 3);
}

#[test]
fn test_widget_tree_flatten() {
    let tw1 = TextWidget::new("A");
    let tw2 = TextWidget::new("B");
    let tree = WidgetTree::new("root")
        .add_child(WidgetTree::leaf("a", &tw1, Rect::new(0, 0, 10, 1)))
        .add_child(WidgetTree::leaf("b", &tw2, Rect::new(0, 1, 10, 1)));
    let cmds = tree.flatten();
    assert_eq!(cmds.len(), 2);
}

#[test]
fn test_widget_tree_find() {
    let tw = TextWidget::new("X");
    let tree = WidgetTree::new("root").add_child(
        WidgetTree::new("container").add_child(WidgetTree::leaf(
            "deep",
            &tw,
            Rect::new(0, 0, 5, 1),
        )),
    );
    assert!(tree.find("deep").is_some());
    assert!(tree.find("nonexistent").is_none());
    assert!(tree.find("root").is_some());
}

#[test]
fn test_widget_tree_with_commands() {
    let cmds = vec![RenderCommand::Clear {
        rect: Rect::new(0, 0, 10, 5),
    }];
    let tree = WidgetTree::new("test").with_commands(cmds.clone());
    assert_eq!(tree.commands.len(), 1);
}

#[test]
fn test_widget_tree_flatten_nested() {
    let inner_cmds = vec![RenderCommand::DrawText {
        x: 0,
        y: 0,
        text: "inner".to_string(),
        style: TextStyle::default(),
    }];
    let outer_cmds = vec![RenderCommand::Clear {
        rect: Rect::new(0, 0, 10, 5),
    }];
    let tree = WidgetTree::new("root")
        .with_commands(outer_cmds)
        .add_child(WidgetTree::new("child").with_commands(inner_cmds));
    let flat = tree.flatten();
    assert_eq!(flat.len(), 2);
}

#[test]
fn test_widget_tree_find_nested_not_found() {
    let tree = WidgetTree::new("root").add_child(WidgetTree::new("child"));
    assert!(tree.find("nonexistent").is_none());
}

#[test]
fn test_widget_tree_node_count_deep() {
    let tree = WidgetTree::new("root")
        .add_child(
            WidgetTree::new("a")
                .add_child(WidgetTree::new("a1"))
                .add_child(WidgetTree::new("a2")),
        )
        .add_child(WidgetTree::new("b"));
    assert_eq!(tree.node_count(), 5);
}

struct TestApp {
    quit: bool,
}

impl TuiApp for TestApp {
    fn init(&mut self) {
        self.quit = false;
    }

    fn update(&mut self, event: Event) -> EventResult {
        if let Event::Key(KeyEvent {
            code: KeyCode::Esc, ..
        }) = event
        {
            self.quit = true;
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }

    fn view(&self) -> WidgetTree {
        let tw = TextWidget::new("Test App");
        WidgetTree::leaf("main", &tw, Rect::new(0, 0, 80, 24))
    }

    fn should_quit(&self) -> bool {
        self.quit
    }
}

#[test]
fn test_tui_app_lifecycle() {
    let mut app = TestApp { quit: false };
    app.init();
    assert!(!app.should_quit());

    let tree = app.view();
    assert_eq!(tree.id, "main");

    let result = app.update(Event::Key(KeyEvent::new(KeyCode::Char('a'))));
    assert_eq!(result, EventResult::Ignored);
    assert!(!app.should_quit());

    let result = app.update(Event::Key(KeyEvent::new(KeyCode::Esc)));
    assert_eq!(result, EventResult::Consumed);
    assert!(app.should_quit());
}
