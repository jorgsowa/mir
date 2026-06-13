===description===
Detect implicit void return
===config===
suppress=MissingClosureReturnType
===file===
<?php
/**
 * @param Closure():Exception $c
 */
function takesClosureReturningException(Closure $c) : void {
    echo $c()->getMessage();
}

takesClosureReturningException(
    function () {
        echo "hello";
    }
);
===expect===
