===description===
an offset write on an object variable uses it; on a local array it does not
===file===
<?php
function f(): void {
    $bag = new ArrayObject();
    $bag['k'] = 1;

    $arr = [];
//  ^^^^ UnusedVariable: Variable $arr is never read
    $arr['k'] = 1;
}
