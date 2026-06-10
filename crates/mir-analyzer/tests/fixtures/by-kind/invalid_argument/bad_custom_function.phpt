===description===
Bad custom function
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
