===description===
Negative control for the typed-array-const literal-narrowing fix: an
EMPTY array constant still correctly flags — the fix only stops a
non-empty literal from losing its known element count, it doesn't
special-case `array_key_first()` itself.
===file===
<?php
final class C {
    public const array D = [];
    public static function f(): string {
        return array_key_first(self::D);
    }
}
===expect===
NullableReturnStatement@5:8-5:40: Return type 'string|int|null' is not compatible with declared 'string'
