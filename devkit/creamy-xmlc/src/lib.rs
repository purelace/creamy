#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_possible_truncation)]
//#![deny(clippy::unwrap_used)]
//#![deny(clippy::panic)]
#![deny(clippy::todo)]
//#![deny(clippy::as_conversions)]

mod compiler;
mod diagnostics;
pub mod error;
pub mod model;
mod table;
pub mod tokenizer;
mod tree;
pub mod utils;
mod version;

pub use compiler::compile;
pub use diagnostics::Diagnostics;
pub use model::definition::{Access, ProtocolDefinition};
pub use table::{FinishedTypeTable, TypeId};
pub use tree::nodes::VariantValue;
pub use utils::{StringPoolIntern, StringPoolResolver};

pub mod constraints {
    // Глобальные ограничения:
    //TODO: move to pool crate
    pub const MAX_UNIQUE_STRINGS: usize = 65536;

    /// * dst: u8,
    /// * group: u8,
    /// * src: u8,
    /// * kind: u8,
    pub const HEADER_BYTES: u8 = 4;

    pub const MAX_GROUPS: usize = 255;
    pub const MAX_MESSAGES_PER_GROUP: usize = 255;

    pub const MAX_STRUCTS: usize = 2048;

    pub const MAX_FLAGS: usize = 2048;
    pub const MAX_OPTIONS: usize = 65536;

    pub const MAX_BITSETS: usize = 2048;
    pub const MAX_BITSET_VALUES: usize = 65536;
    pub const MAX_BITSET_SIZE: usize = MAX_PAYLOAD * 8;

    pub const MAX_ENUMS: usize = 2048;
    pub const MAX_VARIANTS: usize = 65536; //REMOVE

    pub const MAX_FIELDS: usize = 2048;
    pub const MAX_FIELD_PER_STRUCT: usize = 28;

    pub const MAX_PAYLOAD: usize = 28;
    pub const MAX_MESSAGES: usize = MAX_GROUPS * MAX_MESSAGES_PER_GROUP;
    pub const MAX_TYPE_COUNT: usize = MAX_STRUCTS + MAX_ENUMS + MAX_BITSETS + MAX_FLAGS;
}

/*
 * TODO: bool type
 * TODO: C Header generation
 *
 * Pipeline:
 *     Tokenizer -> AST
 *     Resolve:
 *         Errors:
 *             Name duplicates
 *             Empty structs, fields, enums, messages, flags, arrays,
 *             Invalid size
 *             Field, variant, option count
 *
 *         Cache size and align
 *
 *
 *
 * Ограничения на протокол:
 *     i8/i16/u8/u16 - enum types
 *     28 полей на структуру
 *     28 свободных байт
 *     255 групп
 *     255 сообщений
 */

//TODO: validate size
//TODO: remove unused
//TODO: errors
//TODO: warnings
//TODO: executable
//TODO: suggest best layout
//TODO: name duplicate
//TODO: infinity reference
//TODO: missing type reference

// Разгребаю все TODO
// Привожу компилятор в порядок
// Доделываю тесты
// Пишу документацию
// Рассмотреть доктесты
// Разделить ошибки на несколько разных Enum
//
// Привожу в порядок CLI утилиту
// Пишу тесты для утилиты
// Пишу документацию для утилиты
//
// Привожу в порядок кодогенерацию
// Сделать нормальное API для него. Возможно имеет смысл вынести кодогенерацию в отдельный процесс.
// Пишу тесты для кодогенерации
//
// Решить вопрос с creamy-protocol
//
// Приступить к написанию загрузчика
// Загрузчик должен подгружать плагин постепенно, а не сразу все.
//
// Пофиксить шину: пусть плагин экспортирует буфер
//

/*
 * Три структуры:
 * AST
 * Model
 * Valid model
 *
 * Разница между valid model и model в том, что model нужна для того чтобы сделать валидацию модели после загрузки с диска.
 */
