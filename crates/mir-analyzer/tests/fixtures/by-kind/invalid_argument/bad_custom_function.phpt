===description===
Bad custom function
===ignore===
TODO
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
