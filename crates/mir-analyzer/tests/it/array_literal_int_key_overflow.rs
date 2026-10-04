//! An array literal whose explicit int key is `PHP_INT_MAX` must not overflow the implicit next key.

use mir_analyzer::test_utils::check;

#[test]
fn explicit_max_int_key_followed_by_implicit_key_does_not_panic() {
    check("<?php\nfunction f(): array { return [PHP_INT_MAX => 1, 2]; }\n");
    check("<?php\nfunction f(): array { return [9223372036854775807 => 1, 2]; }\n");
}
