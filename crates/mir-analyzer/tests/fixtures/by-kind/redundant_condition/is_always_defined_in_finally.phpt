===description===
Is always defined in finally
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
RedundantCondition@17:13-17:23: Condition is always true/false for type 'Exception'
