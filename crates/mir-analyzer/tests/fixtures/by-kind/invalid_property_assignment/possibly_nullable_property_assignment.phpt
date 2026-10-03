===description===
Possibly nullable property assignment
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

$a = rand(0, 10) ? new Foo() : null;

$a->foo = "hello";
===expect===
