use super::*;

pub(crate) fn simple_event_target_add_event_listener_callback<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    args: v8::FunctionCallbackArguments<'s>,
    _rv: v8::ReturnValue<'s, v8::Value>,
) {
    let Some(target) = simple_event_target_receiver(scope, args.this()) else {
        return;
    };
    simple_object_event_target_add_listener_for_target(scope, &args, target);
}

pub(crate) fn simple_event_target_remove_event_listener_callback<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    args: v8::FunctionCallbackArguments<'s>,
    _rv: v8::ReturnValue<'s, v8::Value>,
) {
    let Some(target) = simple_event_target_receiver(scope, args.this()) else {
        return;
    };
    simple_object_event_target_remove_listener_for_target(scope, &args, target);
}

pub(crate) fn simple_event_target_dispatch_event_callback<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    args: v8::FunctionCallbackArguments<'s>,
    mut rv: v8::ReturnValue<'s, v8::Value>,
) {
    let Some(target) = simple_event_target_receiver(scope, args.this()) else {
        rv.set(v8::Boolean::new(scope, true).into());
        return;
    };
    if crate::web_api_interfaces::IDBRequest::is_instance(scope, target)
        || crate::web_api_interfaces::IDBTransaction::is_instance(scope, target)
        || crate::web_api_interfaces::IDBDatabase::is_instance(scope, target)
    {
        crate::context_bootstrap::indexed_db::dispatch_indexed_db_script_event(scope, args, target, rv);
        return;
    }
    simple_object_event_target_dispatch(scope, &args, target, &mut rv);
}
