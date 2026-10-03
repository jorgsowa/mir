===description===
method call
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {
    public function bar(int $n): void { var_dump($n); }
}

$f = new Foo();
$f->bar(null);
//      ^^^^ NullArgument: Argument $n of bar() cannot be null
===expect===
