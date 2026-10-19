use super::SdpType;
use crate::web_api_interfaces;
use crate::{
    util::{callback_data_index_value, callback_data_item, throw_type_error, v8str},
    webidl::{self, WebIdlDictionarySerialize},
};
use moli_webapi_declare::{
    WebApiFunctionTemplate, WebApiValue, initialize_web_api_constructor_receiver,
};

const DESCRIPTION_SLOT: &str = "__moliRtcSessionDescription";
const DESCRIPTION_MEMBERS: &[&str] = &["type", "sdp"];
type DescriptionSnapshot<'s> = webidl::DictionarySnapshot<'s, SessionDescriptionInit>;

#[derive(webidl::WebIdlDictionary)]
#[webidl(prefix = "RTCSessionDescriptionInit", snapshot)]
struct SessionDescriptionInit {
    #[webidl(converter = "raw", default = webidl::DomString16(Vec::new()))]
    #[webidl(snapshot_order = 1)]
    sdp: webidl::DomString16,
    // Input conversion is lexical; the legacy JSON result keeps type before sdp.
    #[webidl(required, converter = "enum")]
    #[webidl(snapshot_order = 0)]
    r#type: SdpType,
}

#[derive(webidl::WebIdlArgs)]
#[webidl(prefix = "RTCSessionDescription")]
struct SessionDescriptionArgs {
    #[webidl(required, dictionary)]
    init: SessionDescriptionInit,
}

impl SessionDescriptionInit {
    fn from_native_parts<'s>(
        scope: &mut v8::PinScope<'s, '_>,
        kind: v8::Local<'s, v8::String>,
        sdp: v8::Local<'s, v8::String>,
    ) -> Self {
        let mut units = vec![0; sdp.length()];
        sdp.write_v2(scope, 0, &mut units, v8::WriteFlags::empty());
        Self {
            r#type: <SdpType as webidl::WebIdlEnum>::parse_token(&kind.to_rust_string_lossy(scope))
                .expect("native RTCSdpType"),
            sdp: webidl::DomString16(units),
        }
    }
}

#[derive(WebApiFunctionTemplate)]
#[webapi(interface = web_api_interfaces::RTCSessionDescription, enumerable, receiver)]
struct SessionDescriptionPrototypeDeclaration {
    #[webapi(accessor_property, getter = description_attribute_getter, data = callback_data_index_value(scope, 0))]
    r#type: (),
    #[webapi(accessor_property, getter = description_attribute_getter, data = callback_data_index_value(scope, 1))]
    sdp: (),
    #[webapi(method = "toJSON", length = 0, callback = description_to_json_callback)]
    to_json: (),
}

pub(super) fn install_session_description_template_bindings<'s>(
    scope: &mut v8::PinScope<'s, '_, ()>,
    template: v8::Local<'s, v8::FunctionTemplate>,
) {
    SessionDescriptionPrototypeDeclaration::initialize_prototype_template(
        scope,
        template.prototype_template(scope),
    );
}

pub(super) fn from_parts<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    kind: v8::Local<'s, v8::String>,
    sdp: v8::Local<'s, v8::String>,
) -> Option<v8::Local<'s, v8::Object>> {
    let object = super::super::exposed_interfaces::build_intrinsic_interface_instance(
        scope,
        "RTCSessionDescription",
    )
    .ok()?;
    let prototype = super::super::exposed_interfaces::ensure_intrinsic_interface_prototype(
        scope,
        "RTCSessionDescription",
    )
    .ok()?;
    if object.set_prototype(scope, prototype.into()) != Some(true) {
        return None;
    }
    SessionDescriptionInit::from_native_parts(scope, kind, sdp)
        .to_snapshot(scope)?
        .store_private(scope, object, DESCRIPTION_SLOT)?;
    web_api_interfaces::RTCSessionDescription::DESCRIPTOR
        .initialize(scope, object)
        .ok()?;
    Some(object)
}

pub(in crate::context_bootstrap) fn rtc_session_description_constructor_callback<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    args: v8::FunctionCallbackArguments<'s>,
    mut rv: v8::ReturnValue<'s, v8::Value>,
) {
    if !args.is_construct_call() {
        throw_type_error(scope, "RTCSessionDescription constructor requires 'new'.");
        return;
    }
    let Some(parsed) = webidl::parse_args::<SessionDescriptionArgs>(scope, &args) else {
        return;
    };
    if !initialize_web_api_constructor_receiver(scope, args.this(), "RTCSessionDescription") {
        return;
    }
    let Some(snapshot) = parsed.init.to_snapshot(scope) else {
        return;
    };
    if snapshot
        .store_private(scope, args.this(), DESCRIPTION_SLOT)
        .is_none()
    {
        return;
    }
    rv.set(args.this().into());
}

fn description_attribute_getter<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    args: v8::FunctionCallbackArguments<'s>,
    mut rv: v8::ReturnValue<'s, v8::Value>,
) {
    let Some(member) = callback_data_item(
        scope,
        &args,
        DESCRIPTION_MEMBERS,
        "RTCSessionDescription attribute",
    ) else {
        return;
    };
    let record = snapshot(scope, args.this())
        .to_v8_value(scope)
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .expect("native RTCSessionDescription record");
    if let Some(value) = record.get(scope, v8str(scope, member).into()) {
        rv.set(value);
    }
}

fn snapshot<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    receiver: v8::Local<'s, v8::Object>,
) -> DescriptionSnapshot<'s> {
    let target = moli_webapi_declare::web_api_object_target(scope, receiver)
        .expect("validated RTCSessionDescription receiver");
    DescriptionSnapshot::from_private_slot(scope, target, DESCRIPTION_SLOT)
        .expect("native RTCSessionDescription snapshot")
}

fn description_to_json_callback<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    args: v8::FunctionCallbackArguments<'s>,
    mut rv: v8::ReturnValue<'s, v8::Value>,
) {
    if let Some(json) =
        snapshot(scope, args.this()).to_js_snapshot(scope, webidl::SnapshotPrototype::Ordinary)
    {
        rv.set(json.into());
    }
}
