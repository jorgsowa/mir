===description===
conditional types with different branches are not simplified
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class TestFactory {
//<^^^^^^^^^^^^^^^^^^^ MissingConstructor: Class TestFactory has uninitialized properties but no constructor
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
//<^^^^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $intProp expects 'int', cannot assign 'string'
