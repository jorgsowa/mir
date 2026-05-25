===description===
conditional types with different branches are not simplified
===ignore===
===file===
<?php
class TestFactory {
    /**
     * Returns different types based on condition
     * @return ($x is null ? string : int)
     */
    public function process($x) {}

    public string $stringProp;

    public int $intProp;
}

$f = new TestFactory();
$result = $f->process(null);

// Result could be string or int, so assigning to either requires narrowing
$f->stringProp = $result;
$f->intProp = $result;
===expect===
InvalidPropertyAssignment@18:1: Property $stringProp expects 'string', cannot assign 'int|string'
InvalidPropertyAssignment@19:1: Property $intProp expects 'int', cannot assign 'int|string'
