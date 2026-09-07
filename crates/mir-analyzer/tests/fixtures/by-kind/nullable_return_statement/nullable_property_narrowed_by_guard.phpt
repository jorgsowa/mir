===description===
FALSE POSITIVE reproducer. Valid PHP: `?int` property is non-null inside an `if ($this->id !== null)` guard.
mir 0.42.0 currently emits (the bug): NullableReturnStatement@7:12-7:29: expected int, actual int|null
Expected: no issue. Remove ===ignore=== to activate once fixed.
===config===
php_version=8.4
===file===
<?php
class Box {
    private ?int $id = null;
    public function get(): int {
        // FP expected: NullableReturnStatement (?int not narrowed after !== null guard)
        if ($this->id !== null) {
            return $this->id;
        }
        return 0;
    }
}
===expect===
