===description===
No missing-return error when try body returns and all catch blocks also diverge
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function alwaysReturns(): bool {
    try {
        return true;
    } catch (\Exception $e) {
        throw $e;
    }
}

function withFinally(): bool {
    try {
        return true;
    } catch (\Exception $e) {
        throw $e;
    } finally {
        echo "cleanup";
    }
}

function noReturnStillErrors(): bool {
//                              ^^^^ InvalidReturnType: Return type 'void' is not compatible with declared 'bool'
    try {
        echo "hello";
    } catch (\Exception $e) {
        throw $e;
    }
}
