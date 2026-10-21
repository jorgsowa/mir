===description===
an offset write on an object parameter uses it; on an array parameter it does not
===file===
<?php
function fill(ArrayObject $bag): void {
    $bag['k'] = 1;
    $bag['a']['b'] = 2;
}

function local(array $arr): void {
    $arr['k'] = 1;
}
===expect===
UnusedParam@7:15-7:25: Parameter $arr is never used
