===description===
Array filter callback param is seeded as int, so a method call on it is InvalidMethodCall
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = array_filter([1, 2, 3, 4], function ($i) { return $i->foo(); });
//                                                     ^^^^^^^^^ InvalidMethodCall: Cannot call method foo() on non-object type '1|2|3|4'
