===description===
This var with bad type
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    /** @var int */
    public $a = 0;

    /** @var string */
    public $b = "";

    public function fooFoo(): string
    {
        list($this->a, $this->b) = ["a", "b"];
//           ^^^^^^^^ InvalidPropertyAssignment: Property $a expects 'int', cannot assign '"a"'

        return $this->a;
//      ^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'int' is not compatible with declared 'string'
    }
}
===expect===
