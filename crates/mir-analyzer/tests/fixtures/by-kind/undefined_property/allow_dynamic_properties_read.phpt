===description===
Reading an undeclared property of an `#[AllowDynamicProperties]` class or its subclass is not reported.
===file===
<?php
#[\AllowDynamicProperties]
class Ext {}
class Sub extends Ext {}
class Plain extends stdClass {}

function f(Ext $e, Sub $s, Plain $p): void {
    echo $e->status;
    echo $s->sort;
    echo $p->x;
}
