===description===
ReadonlyPropertyAssignment when a subclass's own constructor writes to a readonly property declared on the parent.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {
    public readonly string $name;
}

class Child extends Base {
    public function __construct(string $name) {
        $this->name = $name;
//      ^^^^^^^^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Base::$name outside of constructor
    }
}
===expect===
