===description===
new with object variable is valid PHP (constructs a fresh instance of that
object's own runtime class) and must not error
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {}

function test(Foo $obj) {
    new $obj();
}
