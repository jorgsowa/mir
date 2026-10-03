===description===
NoInterfaceProperties fires on a property write through a plain interface type
without @seal-properties too — the write-side check (expr/assignment.rs) mirrors
the read-side one.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Shape {
    public function area(): float;
}

class Circle implements Shape {
    public float $radius = 1.0;
    public function area(): float {
        return 3.14 * $this->radius * $this->radius;
    }
}

function resize(Shape $s, float $value): void {
    $s->radius = $value;
//  ^^^^^^^^^^^^^^^^^^^ NoInterfaceProperties: Property $radius is not defined on this interface
}

===expect===
