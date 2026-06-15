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
            echo "here";
        }
    }
}
===expect===
RedundantCondition@17:12-17:22: Condition is always true/false for type 'Exception'
