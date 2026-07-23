===description===
`unset($this->items)` mutates a readonly property just as much as a plain
assignment does (PHP itself forbids unsetting a readonly property from any
scope), but unset() only ever ran the purity-only helper, never the
readonly check.
===config===
suppress=MissingConstructor
===file===
<?php
class Box {
    public readonly array $items;

    public function clear(): void {
        unset($this->items);
    }
}
===expect===
ReadonlyPropertyAssignment@6:14-6:26: Cannot assign to readonly property Box::$items outside of constructor
