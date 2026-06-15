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
UndefinedFunction@10:4-10:8: Function ff() is not defined
