===description===
partially nested conditionals (inner simplifies, outer branches differ)
===ignore===
===file===
<?php
class PartialFactory {
    /**
     * Inner conditional has identical branches, but outer branches differ
     * @return ($x is null ? ($y is int ? string : string) : int)
     */
    public function makePartial($x, $y) {}

    public string $stringProp;

    public int $intProp;
}

$factory = new PartialFactory();
// Inner conditional (Y is int ? string : string) -> string
// Result: (X is null ? string : int) -> string|int
$result = $factory->makePartial(null, 1);
$factory->stringProp = $result;
$factory->intProp = $result;
===expect===
InvalidPropertyAssignment@19:1: Property $stringProp expects 'string', cannot assign 'int|string'
InvalidPropertyAssignment@20:1: Property $intProp expects 'int', cannot assign 'int|string'
