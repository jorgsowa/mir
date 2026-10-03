===description===
Array map use method on inferable int
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
//                                    ^^^^^^^^^ MixedMethodCall: Method foo() called on mixed type
===expect===
