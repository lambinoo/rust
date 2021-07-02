// run pass

#![feature(const_option)]

const X: Option<i32> = Some(21);
const Y: Option<i32> = Some(42);
const NONE: Option<i32> = None;

// Tests for Option::ok_or
const SOME_OK_OR: Result<i32, bool> = X.ok_or(false);
const NONE_OK_OR: Result<i32, bool> = NONE.ok_or(false);

// Tests for Option::xor
const SOME_XOR_SOME: Option<i32> = X.xor(Y);
const SOME_XOR_NONE: Option<i32> = X.xor(NONE);
const NONE_XOR_SOME: Option<i32> = NONE.xor(X);
const NONE_XOR_NONE: Option<i32> = NONE.xor(NONE);

// Tests for Option::and
const SOME_AND_SOME: Option<i32> = X.and(Y);
const SOME_AND_NONE: Option<i32> = X.and(NONE);
const NONE_AND_SOME: Option<i32> = NONE.and(X);
const NONE_AND_NONE: Option<i32> = NONE.and(NONE);


fn main() {
    assert_eq!(SOME_OK_OR, X);
    assert_eq!(NONE_OK_OR, Err(false));

    assert_eq!(SOME_XOR_SOME, None);
    assert_eq!(SOME_XOR_NONE, X);
    assert_eq!(NONE_XOR_SOME, X);
    assert_eq!(NONE_XOR_NONE, NONE);

    assert_eq!(SOME_AND_SOME, Y);
    assert_eq!(SOME_AND_NONE, NONE);
    assert_eq!(NONE_AND_SOME, NONE);
    assert_eq!(NONE_AND_NONE, NONE);
}

