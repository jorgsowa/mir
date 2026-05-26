===description===
string interpolation not reported
===file===
<?php
function foo(): string {
    $name = 'world';
    return "Hello $name!";
}
===expect===
