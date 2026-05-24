===description===
isAlwaysDefinedInFinally
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
RedundantCondition
===ignore===
TODO
