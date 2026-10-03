===description===
!isset short-circuit with || operator — function call with narrowed variable
Variable passed as function argument on RHS should be narrowed from !isset() in LHS
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function doSomething(object $obj): bool { return true; }
//                   ^^^^^^^^^^^ UnusedParam: Parameter $obj is never used
if (!isset($x) || doSomething($x)) {
    // After fix: $x in function call should be narrowed as defined
}
===expect===
