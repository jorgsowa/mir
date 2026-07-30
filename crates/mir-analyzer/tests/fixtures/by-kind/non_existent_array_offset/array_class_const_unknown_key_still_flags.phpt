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
    }
}
===expect===
MixedReturnStatement@5:8-5:33: Cannot return a mixed type from function with declared return type 'int'
NonExistentArrayOffset@5:25-5:31: Array offset 'nope' does not exist
