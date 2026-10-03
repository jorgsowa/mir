===description===
Enum wrong float
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Ns;

/** @param 1.2|3.4|5.6 $s */
function foo($s) : void {}
foo(7.8);
//  ^^^ InvalidArgument: Argument $s of foo() expects '1.2|3.4|5.6', got '7.8'
===expect===
