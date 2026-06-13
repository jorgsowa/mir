===description===
No missing-return error when try body returns and all catch blocks also diverge
===config===
suppress=MissingThrowsDocblock
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
    try {
        echo "hello";
    } catch (\Exception $e) {
        throw $e;
    }
}
===expect===
InvalidReturnType@20:38-26:39: Return type 'void' is not compatible with declared 'bool'
