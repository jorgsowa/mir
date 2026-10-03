===description===
ReadonlyPropertyAssignment when child class method tries to set parent's readonly property
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
    public function init(string $name): void {
        $this->name = $name;
//      ^^^^^^^^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Base::$name outside of constructor
    }
}
===expect===
