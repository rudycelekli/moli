use super::*;
use crate::context_bootstrap::shared_event_targets::shared_target_owner;
use crate::util::{get_private_object, new_null_prototype_object, set_private_value};

const DISPATCH_IDENTITY_SLOT: &str = "__moliSimpleEventTargetDispatchIdentity";

pub(crate) fn bind_simple_event_target_dispatch_identity<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    target: v8::Local<'s, v8::Object>,
    identity: v8::Local<'s, v8::Object>,
) {
    let target = moli_webapi_declare::web_api_object_target(scope, target).unwrap_or(target);
    set_private_value(scope, target, DISPATCH_IDENTITY_SLOT, identity.into());
}

pub(crate) fn initialize_simple_event_target<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    target: v8::Local<'s, v8::Object>,
) {
    let target = shared_target_owner(scope, target);
    if get_private_object(scope, target, SIMPLE_EVENT_TARGET_LISTENERS_SLOT).is_none() {
        let registry = new_null_prototype_object(scope);
        set_private_value(
            scope,
            target,
            SIMPLE_EVENT_TARGET_LISTENERS_SLOT,
            registry.into(),
        );
    }
}

pub(crate) fn is_simple_event_target<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    target: v8::Local<'s, v8::Object>,
) -> bool {
    simple_event_target_receiver(scope, target).is_some()
}

pub(super) fn simple_event_target_receiver<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    target: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    // Resolve registered native Proxies only for the simple-target branch.
    // DOM and Window targets keep their existing receiver resolution.
    let native_target = if target.is_proxy() {
        moli_webapi_declare::web_api_object_target(scope, target)?
    } else {
        target
    };
    simple_object_event_listener_registry(scope, native_target)?;
    // Producers with a fixed exposed object can bind that identity separately
    // from the registry. Other native Proxy surfaces dispatch on their wrapper.
    Some(get_private_object(scope, native_target, DISPATCH_IDENTITY_SLOT).unwrap_or(target))
}

pub(super) fn simple_object_event_listener_registry<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    target: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let target = shared_target_owner(scope, target);
    // Read only the target's own private state. Looking through prototypes would
    // let instances inherit another target's listener list and receiver identity.
    get_private_object(scope, target, SIMPLE_EVENT_TARGET_LISTENERS_SLOT)
}
