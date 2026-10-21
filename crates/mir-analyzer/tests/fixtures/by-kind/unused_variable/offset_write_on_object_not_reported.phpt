===description===
an offset write on an object variable uses it; on a local array it does not
===file===
<?php
function f(): void {
    $bag = new ArrayObject();
    $bag['k'] = 1;

    $arr = [];
    $arr['k'] = 1;
}
===expect===
UnusedVariable@6:4-6:8: Variable $arr is never read
