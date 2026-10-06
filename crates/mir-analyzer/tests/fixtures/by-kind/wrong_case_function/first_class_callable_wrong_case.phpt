===description===
Wrong case in first-class callable syntax is reported.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function myFunc(int $x): int { return $x; }

$fn = MYFUNC(...);
//    ^^^^^^ WrongCaseFunction: Function name 'MYFUNC' has incorrect casing; use 'myFunc'
