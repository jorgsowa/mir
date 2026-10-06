===description===
bare generic property still rejects value of completely different class
===file===
<?php
/** @template T */
class ProphecyA {}

/** @template T */
class ProphecyB {}

class Holder {
//<^^^^^^^^^^^^^^ MissingConstructor: Class Holder has uninitialized properties but no constructor
    public ProphecyA $prop;
}

$h = new Holder();
/** @var ProphecyB<string> $b */
$b = new ProphecyB();
$h->prop = $b;
//<^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $prop expects 'ProphecyA', cannot assign 'ProphecyB<string>'
