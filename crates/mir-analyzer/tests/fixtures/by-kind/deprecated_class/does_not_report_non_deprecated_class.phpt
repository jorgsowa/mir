===description===
does not report non deprecated class
===file===
<?php
class ActiveClass {}

function test(): void {
    $obj = new ActiveClass();
}
===expect===
UnusedVariable@5:5-5:9: Variable $obj is never read
