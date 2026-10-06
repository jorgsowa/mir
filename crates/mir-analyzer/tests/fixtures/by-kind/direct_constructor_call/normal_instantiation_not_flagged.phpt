===description===
DirectConstructorCall does NOT fire for normal object instantiation.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {
    public function __construct(int $x) {}
}

$obj = new Foo(1);
