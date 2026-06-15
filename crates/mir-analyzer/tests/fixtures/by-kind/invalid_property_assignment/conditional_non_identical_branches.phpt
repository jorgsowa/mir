===description===
conditional types with different branches are not simplified
===config===
suppress=UnusedParam
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
MissingConstructor@2:0-2:19: Class TestFactory has uninitialized properties but no constructor
InvalidPropertyAssignment@19:0-19:21: Property $intProp expects 'int', cannot assign 'string'
