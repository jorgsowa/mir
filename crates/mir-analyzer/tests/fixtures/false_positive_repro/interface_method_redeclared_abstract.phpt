===description===
FALSE POSITIVE reproducer. Valid PHP: Interface methods are implicitly abstract; an abstract class may re-declare them `abstract`.
mir 0.42.0 currently emits (the bug): MethodSignatureMismatch@8:4-8:52: cannot make non-abstract method abstract
Expected: no issue. Remove ===ignore=== to activate once fixed.
===config===
php_version=8.4
===file===
<?php
interface Writer {
    public function write(string $s): void;
}
abstract class BaseWriter implements Writer {
    // FP expected: MethodSignatureMismatch ("cannot make non-abstract method abstract"
    // — interface methods are implicitly abstract)
    abstract public function write(string $s): void;
}
===expect===
