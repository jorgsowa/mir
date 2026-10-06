===description===
no type not reported
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {
    public $name;
}

$f = new Foo();
$f->name = 42;
