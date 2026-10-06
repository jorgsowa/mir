===description===
Mixed argument
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function fooFoo(int $a): void {}
/** @var mixed */
$a = "hello";
fooFoo($a);
//     ^^ MixedArgument: Argument $a of fooFoo() is mixed
