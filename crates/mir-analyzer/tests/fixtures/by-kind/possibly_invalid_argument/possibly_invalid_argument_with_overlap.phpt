===description===
Possibly invalid argument with overlap
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {}
class B {}
class C {}

$foo = rand(0, 1) ? new A : new B;

/** @param B|C $b */
function bar($b) : void {}

bar($foo);
//  ^^^^ PossiblyInvalidArgument: Argument $b of bar() expects 'B|C', possibly different type 'A|B' provided
