===description===
Type coercion
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {}
class B extends A{}

function fooFoo(B $b): void {}
fooFoo(new A());
//     ^^^^^^^ ArgumentTypeCoercion: Argument $b of fooFoo() expects 'B', got 'A' — coercion may fail at runtime
