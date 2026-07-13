===description===
Bad custom function
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param callable(int):bool $func
 */
function takesFunction(callable $func) : void {}

function myFunction( string $foo ) : bool {
    return false;
}

takesFunction("myFunction");
===expect===
InvalidArgument@11:14-11:26: Argument $func of takesFunction() expects 'callable whose parameter #1 accepts int', got 'callable whose parameter #1 only accepts string'
