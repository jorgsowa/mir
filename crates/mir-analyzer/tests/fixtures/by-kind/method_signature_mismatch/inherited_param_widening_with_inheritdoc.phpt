===description===
`{@inheritDoc}` should inherit the parent docblock; parameter widening is contravariant-legal anyway.
===config===
suppress=UnusedParam
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
