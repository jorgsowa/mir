===description===
new with class-string variable should not error
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {}

function test(string $className) {
    /** @var class-string<Foo> $className */
    new $className();
}
