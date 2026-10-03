===description===
resetAsLazyGhostWithBadType_2
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->resetAsLazyGhost(new Foo, function (Bar $foo) {});
===expect===
