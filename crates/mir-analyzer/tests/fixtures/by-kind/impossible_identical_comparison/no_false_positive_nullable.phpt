===description===
A nullable string includes null — comparison against null should not fire.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(?string $s): void {
    if ($s === null) {}
    if ($s === "hello") {}
}
===expect===
