===description===
bare generic property still rejects value of completely different class
===file===
<?php
/** @template T */
class ProphecyA {}

/** @template T */
class ProphecyB {}

class Holder {
    public ProphecyA $prop;
}

$h = new Holder();
/** @var ProphecyB<string> $b */
$b = new ProphecyB();
$h->prop = $b;
===expect===
InvalidPropertyAssignment@15:1-15:14: Property $prop expects 'ProphecyA', cannot assign 'ProphecyB<string>'
