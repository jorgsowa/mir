===description===
detectUnusedVariableInsideLoopAfterAssignment
===file===
<?php
function foo() : void {
    foreach ([1, 2, 3] as $i) {
        $i = $i;
    }
}
===expect===
UnusedForeachValue
===ignore===
TODO
