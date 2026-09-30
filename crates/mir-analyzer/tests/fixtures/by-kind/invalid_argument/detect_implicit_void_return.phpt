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
//  ^ +2:5 InvalidArgument: Argument $c of takesClosureReturningException() expects 'callable returning Exception', got 'callable returning void'
        echo "hello";
    }
);
===expect===
