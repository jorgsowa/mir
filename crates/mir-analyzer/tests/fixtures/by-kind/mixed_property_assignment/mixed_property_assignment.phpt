===description===
Mixed property assignment
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {
    /** @var string */
    public $foo = "";
}

/** @var mixed */
$a = (new Foo());

$a->foo = "hello";
//<^^^^^^^^^^^^^^^^^ MixedPropertyAssignment: Property $foo assigned on mixed type
