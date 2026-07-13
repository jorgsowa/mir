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
InvalidArgument@10:4-12:5: Argument $callback of typed_callable() expects 'callable returning 'Exception'', got 'callable returning 'void''
