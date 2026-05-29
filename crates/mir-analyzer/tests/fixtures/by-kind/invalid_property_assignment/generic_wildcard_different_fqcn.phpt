===description===
bare generic does not accept different FQCN (strict FQCN matching)
===file===
<?php
/** @template T */
class GenericA {}

/** @template T */
class GenericB {}

class Config {
    public GenericA $a;
}

$c = new Config();
$a = new GenericA();
$c->a = $a;
// This should error: GenericB value cannot assign to GenericA property
$c->a = new GenericB();
===expect===
InvalidPropertyAssignment@16:1-16:23: Property $a expects 'GenericA', cannot assign 'GenericB'
