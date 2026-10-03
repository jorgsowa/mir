===description===
Bad assignment
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    /** @var string */
    public $foo;

    public function barBar(): void
    {
        $this->foo = 5;
//      ^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $foo expects 'string', cannot assign '5'
    }
}
===expect===
