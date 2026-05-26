===description===
Dont qualify string callables
===file===
<?php
namespace NS;

function ff() : void {}

function run(callable $f) : void {
    $f();
}

run("ff");
===expect===
UndefinedFunction
===ignore===
TODO
