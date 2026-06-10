===description===
More specific callable
===ignore===
TODO
===file===
<?php
/** @param callable(string):void $c */
function takesSpecificCallable(callable $c) : void {
    $c("foo");
}

function takesCallable(callable $c) : void {
    takesSpecificCallable($c);
}
===expect===
