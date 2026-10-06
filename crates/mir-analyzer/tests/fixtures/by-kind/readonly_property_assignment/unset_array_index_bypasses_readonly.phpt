===description===
Negative control for the K7 fix: `unset($this->items[$k])` on a PLAIN array
readonly property (not an ArrayAccess object) still mutates the property's
own contents in place — the offset-write exemption only applies when the
property actually holds an ArrayAccess-implementing object.
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

    public function drop(string $k): void {
        unset($this->items[$k]);
//            ^^^^^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Box::$items outside of constructor
    }
}
