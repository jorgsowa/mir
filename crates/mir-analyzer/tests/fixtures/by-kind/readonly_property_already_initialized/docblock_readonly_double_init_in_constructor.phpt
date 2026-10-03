===description===
A `@readonly`-only (non-native) property assigned twice in its own
constructor is flagged the same way a native `readonly` property is — the
docblock tag makes the same "write once" contract, just unenforced by PHP
itself.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Point {
    /** @readonly */
    public $x;

    public function __construct(int $x) {
        $this->x = $x;
        $this->x = $x + 1;
//      ^^^^^^^^^^^^^^^^^ ReadonlyPropertyAlreadyInitialized: Cannot modify readonly property Point::$x — already initialized
    }
}
===expect===
