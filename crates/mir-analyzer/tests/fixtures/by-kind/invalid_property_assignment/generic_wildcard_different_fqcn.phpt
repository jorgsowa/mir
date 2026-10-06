===description===
bare generic does not accept different FQCN (strict FQCN matching)
===file===
<?php
/** @template T */
class GenericA {}

/** @template T */
class GenericB {}

class Config {
//<^^^^^^^^^^^^^^ MissingConstructor: Class Config has uninitialized properties but no constructor
    public GenericA $a;
}

$c = new Config();
$a = new GenericA();
$c->a = $a;
// This should error: GenericB value cannot assign to GenericA property
$c->a = new GenericB();
//<^^^^^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $a expects 'GenericA', cannot assign 'GenericB'
