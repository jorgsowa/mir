===description===
detectUnusedVariableInsideLoopAfterAssignmentWithAddition
===file===
<?php
function foo() : void {
    foreach ([1, 2, 3] as $i) {
        $i = $i + 1;
    }
}
===expect===
UnusedForeachValue
===ignore===
TODO
