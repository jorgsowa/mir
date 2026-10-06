===description===
MixedPropertyFetch does NOT fire when the object has a concrete type.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {
    public string $name = "hello";
}

$obj = new Foo();
$x = $obj->name;
