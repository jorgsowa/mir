===description===
Array filter use method on inferable int
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
//                                                     ^^^^^^^^^ MixedMethodCall: Method foo() called on mixed type
===expect===
