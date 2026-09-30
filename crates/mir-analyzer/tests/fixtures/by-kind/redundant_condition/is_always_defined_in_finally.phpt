===description===
Is always defined in finally
===config===
suppress=MissingThrowsDocblock,UnusedVariable
===file===
<?php
function maybeThrows() : void {
    if (rand(0, 1)) {
        throw new UnexpectedValueException();
    }
}

function doTry() : void {
    $exception = new Exception();

    try {
        maybeThrows();
        return;
    } catch (Exception $exception) {
        throw $exception;
    } finally {
        if ($exception) {
//          ^^^^^^^^^^ RedundantCondition: Condition is always true, so the else branch is never reached
            echo "here";
        }
    }
}
===expect===
