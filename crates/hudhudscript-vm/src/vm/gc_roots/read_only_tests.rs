use super::*;
use hudhudscript_bytecode::{gc, Value16};

fn dyn_value(label: &str) -> Value16 {
    Value16::string(format!("gc-read-only-{}-dynamic-value", label))
}

#[test]
fn mark_from_roots_preserves_iterator_contents_and_position() {
    let mut vm = VM::new();
    let first = dyn_value("iterator-first");
    let second = dyn_value("iterator-second");
    let original = (vec![first, second], "item".to_string(), 1usize);
    vm.iterators.push(original.clone());

    vm.mark_from_roots();

    assert!(gc::is_marked(first));
    assert!(gc::is_marked(second));
    assert_eq!(vm.iterators.len(), 1);
    assert_eq!(vm.iterators[0], original);
}

#[test]
fn mark_from_roots_preserves_actor_mailbox_order_and_reply_target() {
    let mut vm = VM::new();
    let first = dyn_value("mailbox-first");
    let second = dyn_value("mailbox-second");
    let (actor_ref, mailbox) = vm.actors.spawn();
    actor_ref.send(first).expect("send first actor payload");
    actor_ref
        .send_with_reply(second, "reply-target".to_string())
        .expect("send second actor payload");
    vm.actor_mailboxes.insert(actor_ref.id.clone(), mailbox);

    vm.mark_from_roots();

    assert!(gc::is_marked(first));
    assert!(gc::is_marked(second));

    let mailbox = vm
        .actor_mailboxes
        .get(&actor_ref.id)
        .expect("mailbox exists");
    let snapshot = mailbox.peek_nonblocking();
    assert_eq!(snapshot.len(), 2);
    assert_eq!(snapshot[0].payload, first);
    assert_eq!(snapshot[0].reply_to, None);
    assert_eq!(snapshot[1].payload, second);
    assert_eq!(snapshot[1].reply_to.as_deref(), Some("reply-target"));

    let first_msg = mailbox.try_recv().expect("first message preserved");
    let second_msg = mailbox.try_recv().expect("second message preserved");
    assert_eq!(first_msg.payload, first);
    assert_eq!(first_msg.reply_to, None);
    assert_eq!(second_msg.payload, second);
    assert_eq!(second_msg.reply_to.as_deref(), Some("reply-target"));
    assert!(mailbox.try_recv().is_none());
}

#[test]
fn mark_from_roots_marks_active_frame_temporary_without_relocating_frame() {
    let mut vm = VM::new();
    vm.registers.advance(32);
    let base_before = vm.registers.base();
    let len_before = vm.registers.len();
    let rooted = dyn_value("active-frame-temp");
    vm.registers[17] = rooted;

    vm.mark_from_roots();

    assert!(gc::is_marked(rooted));
    assert_eq!(vm.registers[17], rooted);
    assert_eq!(vm.registers.base(), base_before);
    assert_eq!(vm.registers.len(), len_before);
}
