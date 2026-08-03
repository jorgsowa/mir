===description===
M12: `$arr['k'] = v` through a by-ref param is a write observable by the
caller through the reference, same as a plain `$x = v;` overwrite — it
must count as the param being used, not flagged UnusedParam. A plain
local (non-byref) array's offset-write must still NOT count as a read,
so an unrelated dead `UnusedVariable` write stays correctly detected.
===file===
<?php
function setArr(array &$a): void {
    $a['k'] = 5;
}

function localArrNotByref(): void {
    $a = [];
    $a[0] = 1;
}
===expect===
UnusedVariable@7:4-7:6: Variable $a is never read
