===description===
Negative control for the array-class-const shape-inference fix: a key not
present in the constant's own literal shape still correctly flags — the
fix only stops the shape from being discarded, it doesn't widen access
to arbitrary keys.
===file===
<?php
final class C {
    public const MAP = ['a' => 1, 'b' => 2];
    public static function g(): int {
        return self::MAP['nope'];
//      ^^^^^^^^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
//                       ^^^^^^ NonExistentArrayOffset: Array offset 'nope' does not exist
    }
}
===expect===
