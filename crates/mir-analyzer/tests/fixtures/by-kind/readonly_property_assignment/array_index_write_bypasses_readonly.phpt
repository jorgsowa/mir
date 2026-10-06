===description===
`$this->items[] = $n` mutates a readonly array property's contents just
as much as a plain assignment does, but the array-index-write arm only
ever ran the purity-only helper, never the readonly check.
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

    public function push(int $n): void {
        $this->items[] = $n;
//      ^^^^^^^^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Box::$items outside of constructor
    }
}
