===description===
an offset write on an object parameter uses it; on an array parameter it does not
===file===
<?php
function fill(ArrayObject $bag): void {
    $bag['k'] = 1;
    $bag['a']['b'] = 2;
}

function local(array $arr): void {
//             ^^^^^^^^^^ UnusedParam: Parameter $arr is never used
    $arr['k'] = 1;
}
