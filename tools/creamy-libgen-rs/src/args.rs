use creamy_libgen::Target;

#[derive(Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
    pub eq: bool,
    pub ord: bool,
    pub hash: bool,
    pub debug_asserts: bool,
    pub target: Target,
    pub creamy_sdk_path: &'static str,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            eq: true,
            ord: true,
            hash: true,
            debug_asserts: true,
            creamy_sdk_path: "creamy_sdk",
            target: Target::Plugin,
        }
    }
}

impl Args {
    #[doc(hidden)]
    #[must_use]
    pub fn sdk() -> Self {
        Self {
            creamy_sdk_path: "crate",
            target: Target::Sdk,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn host() -> Self {
        Self {
            creamy_sdk_path: "creamy::sdk",
            target: Target::Host,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn plugin() -> Self {
        Self {
            creamy_sdk_path: "creamy_sdk",
            target: Target::Plugin,
            ..Default::default()
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn engine() -> Self {
        Self {
            creamy_sdk_path: "creamy_sdk",
            target: Target::Engine,
            ..Default::default()
        }
    }

    pub(crate) fn typed_message_trait_path(&self) -> String {
        format!("{}::message::TypedMessage", self.creamy_sdk_path)
    }

    pub(crate) fn untyped_message_path(&self) -> String {
        format!("{}::message::UntypedMessage", self.creamy_sdk_path)
    }

    pub(crate) fn message_trait_path(&self) -> String {
        format!("{}::message::Message", self.creamy_sdk_path)
    }

    pub(crate) fn message_size_path(&self) -> String {
        format!("{}::defines::MESSAGE_SIZE", self.creamy_sdk_path)
    }

    pub(crate) fn cast_message_fn_path(&self) -> String {
        format!("{}::message::cast_message", self.creamy_sdk_path)
    }

    pub(crate) fn custom_message_handler_trait_path(&self) -> String {
        format!("{}::api::CustomHandler", self.creamy_sdk_path)
    }

    pub(crate) fn stream_id_path(&self) -> String {
        format!("{}::stream::StreamId", self.creamy_sdk_path)
    }

    pub(crate) fn stream_type_path(&self) -> String {
        format!("{}::stream::StreamType", self.creamy_sdk_path)
    }

    pub(crate) fn stream_message_trait_path(&self) -> String {
        format!("{}::stream::StreamMessage", self.creamy_sdk_path)
    }

    pub(crate) fn stream_content_trait(&self) -> String {
        format!("{}::stream::StreamContent", self.creamy_sdk_path)
    }

    pub(crate) fn stream_max_payload_size_path(&self) -> String {
        format!("{}::stream::MAX_STREAM_PAYLOAD", self.creamy_sdk_path)
    }

    pub(crate) fn array_path(&self) -> String {
        format!("{}::stream::Array", self.creamy_sdk_path)
    }

    pub(crate) fn stream_lightweight_content_path(&self) -> String {
        format!("{}::stream::StreamLightweightContent", self.creamy_sdk_path)
    }

    pub(crate) fn binary_writer_trait_path(&self) -> String {
        format!("{}::binary::BinaryWriter", self.creamy_sdk_path)
    }

    pub(crate) fn binary_reader_trait_path(&self) -> String {
        format!("{}::binary::BinaryReader", self.creamy_sdk_path)
    }

    pub(crate) fn binary_write_error_path(&self) -> String {
        format!("{}::binary::WriteError", self.creamy_sdk_path)
    }

    pub(crate) fn binary_read_error_path(&self) -> String {
        format!("{}::binary::ReadError", self.creamy_sdk_path)
    }

    pub(crate) fn binary_write_trait_path(&self) -> String {
        format!("{}::binary::Write", self.creamy_sdk_path)
    }

    pub(crate) fn binary_endianness_trait_path(&self) -> String {
        format!("{}::binary::Endianness", self.creamy_sdk_path)
    }

    pub(crate) fn binary_read_trait_path(&self) -> String {
        format!("{}::binary::Read", self.creamy_sdk_path)
    }

    pub(crate) fn binary_read_derive_path(&self) -> String {
        format!("{}::binary::BinRead", self.creamy_sdk_path)
    }

    pub(crate) fn binary_write_derive_path(&self) -> String {
        format!("{}::binary::BinWrite", self.creamy_sdk_path)
    }
}
