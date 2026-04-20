===source===
<?php
class ActiveClass {}

function test(): void {
    $obj = new ActiveClass();
}
===expect===
UnusedVariable: Variable $obj is never read
