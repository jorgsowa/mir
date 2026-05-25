===description===
Detect unused variable inside loop after assignment
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
