===description===
does not report non deprecated class
===file===
<?php
class ActiveClass {}

function test(): void {
    $obj = new ActiveClass();
//  ^^^^ UnusedVariable: Variable $obj is never read
}
===expect===
