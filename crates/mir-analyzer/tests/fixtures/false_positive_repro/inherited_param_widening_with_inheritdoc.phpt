===description===
FALSE POSITIVE reproducer. Valid PHP: `{@inheritDoc}` should inherit the parent docblock; parameter widening is contravariant-legal anyway.
mir 0.42.0 currently emits (the bug): MethodSignatureMismatch@10:4-10:48
Expected: no issue. Remove ===ignore=== to activate once fixed.
===ignore===
===config===
php_version=8.4
===file===
<?php
interface Manager {
    /** @param non-empty-list<int> $rows */
    public function rename(array $rows): void;
}
class ManagerImpl implements Manager {
    /**
     * {@inheritDoc}
     */
    public function rename(array $rows): void {}
}
===expect===
