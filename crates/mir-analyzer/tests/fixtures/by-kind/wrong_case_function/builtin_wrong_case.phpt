===description===
Calling a built-in function with wrong casing is reported.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = STRLEN("hello");
//   ^^^^^^ WrongCaseFunction: Function name 'STRLEN' has incorrect casing; use 'strlen'
===expect===
