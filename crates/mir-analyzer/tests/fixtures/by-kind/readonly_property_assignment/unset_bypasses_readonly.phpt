===description===
`unset($this->items)` mutates a readonly property just as much as a plain
assignment does (PHP itself forbids unsetting a readonly property from any
scope), but unset() only ever ran the purity-only helper, never the
readonly check.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Box {
    public readonly array $items;

    public function clear(): void {
        unset($this->items);
//            ^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Box::$items outside of constructor
    }
}
===expect===
