===description===
Array map callback param is seeded as int, so a method call on it is InvalidMethodCall
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = array_map(function ($i) { return $i->foo(); }, [1, 2, 3, 4]);
//                                    ^^^^^^^^^ InvalidMethodCall: Cannot call method foo() on non-object type '1|2|3|4'
