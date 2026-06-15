===description===
partially nested conditionals (inner simplifies, outer branches differ)
===config===
suppress=UnusedParam
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
MissingConstructor@2:0-2:22: Class PartialFactory has uninitialized properties but no constructor
InvalidPropertyAssignment@19:0-19:27: Property $intProp expects 'int', cannot assign 'string'
