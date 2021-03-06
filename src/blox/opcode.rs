// Macro to count tt's
// See https://doc.rust-lang.org/reference/macros-by-example.html
// See https://danielkeep.github.io/tlborm/book/blk-counting.html
macro_rules! count_tts {
    () => {0usize};
    ($name:ident) => {1usize};
    ($name:ident , $( $rest:tt)*) => {1usize + count_tts!($($rest)*)};
}

// Macro to generate the opcode constants and the opcode name array
// See https://stackoverflow.com/questions/51577597/how-to-automatically-generate-incrementing-number-identifiers-for-each-implement
macro_rules! ops {
    // End condition
    ($count:expr ;) => {};

    // Match on count with remaining names
    ($count:expr ; $name:ident $(, $rest:tt)*) => {
        // The statement
        pub const $name: u8 = $count;

        // Recursively call and increment the count
        ops!($count + 1; $($rest),*);
    };
    // Match initial case
    ($($names:ident),+) => {

        // Allocate the opcode name translation array and assign the names
        const OPCODES: [&'static str; count_tts!($($names),*)] = [ $(stringify!($names)),+ ];

        ops!(0; $($names),*);
    };
}

// Sadly(?) I can't use an enum for this, because the list has to be exhaustive
// and I store the code as pure u8
ops!(
    OP_CONSTANT,
    OP_NIL,
    OP_TRUE,
    OP_FALSE,
    OP_POP,
    OP_GET_LOCAL,
    OP_SET_LOCAL,
    OP_GET_GLOBAL,
    OP_DEFINE_GLOBAL,
    OP_SET_GLOBAL,
    OP_EQUAL,
    OP_GREATER,
    OP_LESS,
    OP_MODULO,
    OP_ADD,
    OP_SUBTRACT,
    OP_MULTIPLY,
    OP_DIVIDE,
    OP_NOT,
    OP_NEGATE,
    OP_PRINT,
    OP_JUMP_BACK,
    OP_JUMP,
    OP_JUMP_IF_FALSE,
    OP_CALL,
    OP_RETURN
);

/// Returns the name for the given opcode
// TODO: make safe
pub fn get_name(code: u8) -> &'static str {
    OPCODES[code as usize]
}
