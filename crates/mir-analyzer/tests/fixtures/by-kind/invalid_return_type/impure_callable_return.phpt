===description===
Impure callable return
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @pure
 * @return pure-callable():int
 */
function foo(): callable {
    /** @suppress ImpureFunctionCall */
    return function() {
        echo "bar";
//      ^^^^^^^^^^^ ImpureFunctionCall: Calling impure function echo() in a @pure function
        return 1;
    };
}
===expect===
UnusedSuppress@7:18-7:36: Suppress annotation for 'ImpureFunctionCall' is never used
