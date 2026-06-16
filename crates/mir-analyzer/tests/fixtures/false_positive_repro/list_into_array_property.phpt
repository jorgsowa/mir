===description===
FALSE POSITIVE reproducer. Valid PHP: `list<int>` is an `array<array-key,int>`.
mir 0.42.0 currently emits (the bug): InvalidPropertyAssignment@8:8-8:28: expected array<int|string,int>, actual list<int>
Expected: no issue. Remove ===ignore=== to activate once fixed.
===ignore===
===config===
php_version=8.4
===file===
<?php
class Holder {
    /** @var array<array-key, int> */
    private array $data;
    /** @param list<int> $items */
    public function __construct(array $items) {
        // FP expected: InvalidPropertyAssignment (list<int> not seen as array<array-key,int>)
        $this->data = $items;
    }
}
===expect===
