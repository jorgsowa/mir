===description===
bare generic does not accept different FQCN (strict FQCN matching)
===file===
<?php
class Config {
    /** @var GenericA */
    public $a;

    /** @var GenericB<string> */
    public $b;
}

class GenericA<T> {}
class GenericB<T> {}

$c = new Config();
$a = new GenericA();
$c->a = $a;
// This should error: GenericB<string> cannot assign to GenericA
$c->a = new GenericB();
===expect===
InvalidPropertyAssignment@17:1: Property $a expects 'GenericA', cannot assign 'GenericB'
