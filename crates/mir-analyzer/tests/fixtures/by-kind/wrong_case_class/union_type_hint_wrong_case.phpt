===description===
Wrong case class name in a union type hint is reported; correct-case member is not.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {}
class Bar {}
function process(FOO|Bar $x): void {}
//               ^^^ WrongCaseClass: Class name 'FOO' has incorrect casing; use 'Foo'
