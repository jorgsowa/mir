===description===
`$this->box->n++` mutates a chained receiver's readonly property just as
much as `$this->box->n = ...` does, but `check_property_readonly_write` only
resolved a bare-variable receiver (`ctx.get_var`), silently skipping any
receiver that's itself a property access.
===config===
suppress=MissingConstructor
===file===
<?php
class Box {
    public readonly int $n;
}
class Container {
    public Box $box;
}
class Counter {
    public function bump(Container $c): void {
        $c->box->n++;
    }
}
===expect===
ReadonlyPropertyAssignment@10:8-10:18: Cannot assign to readonly property Box::$n outside of constructor
