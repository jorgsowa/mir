===description===
DuplicateClass does NOT fire when a class is declared only once.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {
    public string $bar = '';
}

$obj = new Foo();
