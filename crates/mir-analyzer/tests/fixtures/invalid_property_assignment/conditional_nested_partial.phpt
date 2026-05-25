===description===
partially nested conditionals (inner simplifies, outer branches differ)
===file===
<?php
class PartialFactory {
    /**
     * Inner conditional has identical branches, but outer branches differ
     * @return ($x is null ? ($y is int ? string : string) : int)
     */
    public function makePartial($x, $y) {}

    /** @var string */
    public $stringProp;

    /** @var int */
    public $intProp;
}

$factory = new PartialFactory();
// Inner conditional (Y is int ? string : string) -> string
// Result: (X is null ? string : int) -> string|int
$result = $factory->makePartial(null, 1);
$factory->stringProp = $result;
$factory->intProp = $result;
===expect===
InvalidPropertyAssignment@20:1: Property $stringProp expects 'string', cannot assign 'int|string'
InvalidPropertyAssignment@21:1: Property $intProp expects 'int', cannot assign 'int|string'
===ignore===
