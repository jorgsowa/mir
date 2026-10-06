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
//                ^^^^^^^^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'ImpureFunctionCall' is never used
    return function() {
        echo "bar";
//      ^^^^^^^^^^^ ImpureFunctionCall: Calling impure function echo() in a @pure function
        return 1;
    };
}
===expect===
