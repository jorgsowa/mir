===description===
Reference to an object property does not fire UnsupportedReferenceUsage.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {
    public string $bar = "x";
}

$obj = new Foo();
$ref = &$obj->bar;
