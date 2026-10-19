use super::super::{COMMAND_EVENT_SOURCE_SLOT, TOGGLE_EVENT_SOURCE_SLOT, event_attribute};
use crate::context_bootstrap::events::EventInit;
use crate::document_runtime::DomHandle;
use crate::native_bridge::JsContextHost;
use crate::{web_api_interfaces, webidl};
use moli_webapi_declare::web_api_object_target;
use webidl::WebIdlDictionarySerialize;

/// EventInit members precede the subclass members in lexical order. Convert
/// the entire dictionary before creating either base or payload state.
#[derive(webidl::WebIdlDictionary)]
#[webidl(prefix = "ToggleEventInit", snapshot)]
pub(super) struct ToggleEventInit<'s> {
    #[webidl(inherit)]
    base: EventInit,
    #[webidl(name = "newState", default = webidl::DomString16(Vec::new()), converter = "raw")]
    #[webidl(snapshot_order = 1)]
    new_state: webidl::DomString16,
    #[webidl(name = "oldState", default = webidl::DomString16(Vec::new()), converter = "raw")]
    #[webidl(snapshot_order = 0)]
    old_state: webidl::DomString16,
    #[webidl(nullable, interface = web_api_interfaces::Element, snapshot_slot = TOGGLE_EVENT_SOURCE_SLOT)]
    source: Option<v8::Local<'s, v8::Object>>,
}

impl Default for ToggleEventInit<'_> {
    fn default() -> Self {
        Self {
            base: EventInit::default(),
            new_state: webidl::DomString16(Vec::new()),
            old_state: webidl::DomString16(Vec::new()),
            source: None,
        }
    }
}

#[derive(webidl::WebIdlDictionary)]
#[webidl(prefix = "CommandEventInit", snapshot)]
pub(super) struct CommandEventInit<'s> {
    #[webidl(inherit)]
    base: EventInit,
    #[webidl(default = webidl::DomString16(Vec::new()), converter = "raw")]
    command: webidl::DomString16,
    #[webidl(nullable, interface = web_api_interfaces::Element, snapshot_slot = COMMAND_EVENT_SOURCE_SLOT)]
    source: Option<v8::Local<'s, v8::Object>>,
}

impl Default for CommandEventInit<'_> {
    fn default() -> Self {
        Self {
            base: EventInit::default(),
            command: webidl::DomString16(Vec::new()),
            source: None,
        }
    }
}

pub(super) fn parse_interaction_event_init<'s, T: webidl::WebIdlDictionary<'s> + Default>(
    scope: &mut v8::PinScope<'s, '_>,
    args: &v8::FunctionCallbackArguments<'s>,
    interface: &'static str,
) -> Option<T> {
    let parsed = webidl::dictionary_arg(args, 1, webidl::Context::argument(interface, 2)).and_then(
        |object| match object {
            Some(object) => webidl::parse_dictionary_object(scope, object),
            None => Ok(T::default()),
        },
    );
    match parsed {
        Ok(init) => Some(init),
        Err(error) => {
            webidl::throw_error(scope, &error);
            None
        }
    }
}

impl<'s> ToggleEventInit<'s> {
    pub(super) fn event_flags(&self) -> (bool, bool, bool) {
        (self.base.bubbles, self.base.cancelable, self.base.composed)
    }

    pub(super) fn initialize(
        self,
        scope: &mut v8::PinScope<'s, '_>,
        event: v8::Local<'s, v8::Object>,
    ) {
        self.initialize_snapshot(scope, event)
            .expect("ToggleEvent snapshot");
    }
}

impl<'s> CommandEventInit<'s> {
    pub(super) fn event_flags(&self) -> (bool, bool, bool) {
        (self.base.bubbles, self.base.cancelable, self.base.composed)
    }

    pub(super) fn initialize(
        self,
        scope: &mut v8::PinScope<'s, '_>,
        event: v8::Local<'s, v8::Object>,
    ) {
        self.initialize_snapshot(scope, event)
            .expect("CommandEvent snapshot");
    }
}

fn native_node_identity<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
) -> Option<(*mut JsContextHost, DomHandle)> {
    let object = v8::Local::<v8::Object>::try_from(value).ok()?;
    if !web_api_interfaces::Node::is_instance(scope, object) {
        return None;
    }
    let target = web_api_object_target(scope, object)?;
    let context = target.get_creation_context(scope)?;
    let scope = &mut v8::ContextScope::new(scope, context);
    crate::native_bridge::node_runtime_and_handle_from_object_or_detached(scope, target).ok()
}

fn retargeted_source<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Value>> {
    let source = event_attribute(scope, event, "source")?;
    let Some((source_host, source_node)) = native_node_identity(scope, source) else {
        return Some(source);
    };
    let current_target = event_attribute(scope, event, "currentTarget")?;
    let against = native_node_identity(scope, current_target).and_then(|(host, node)| {
        // Child realms can have different hosts over the same DOM arena.
        std::ptr::eq(
            unsafe { &*host }.dom_host(),
            unsafe { &*source_host }.dom_host(),
        )
        .then_some(node)
    });
    let retargeted = unsafe { &*source_host }
        .dom_host()
        .retarget(source_node, against);
    if retargeted == source_node {
        return Some(source);
    }
    let source_object = v8::Local::<v8::Object>::try_from(source).ok()?;
    unsafe { &mut *source_host }
        .native_bridge_mut()
        .wrap_handle_for_receiver(scope, source_host, source_object, retargeted)
        .map(Into::into)
}

pub(in crate::context_bootstrap) fn interaction_event_source_getter<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    args: v8::FunctionCallbackArguments<'s>,
    mut rv: v8::ReturnValue<'_, v8::Value>,
) {
    // Generated WebIDL bindings validate the receiver. Always retarget the
    // retained original source against native currentTarget at access time.
    let Some(value) = retargeted_source(scope, args.this()) else {
        return;
    };
    let Some(context) = args.this().get_creation_context(scope) else {
        return;
    };
    if let Some(value) =
        crate::context_bootstrap::platform_object_worlds::in_realm(scope, value, context)
    {
        rv.set(value);
    }
}
