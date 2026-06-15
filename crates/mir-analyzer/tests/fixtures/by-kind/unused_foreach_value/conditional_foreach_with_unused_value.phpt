===description===
Conditional foreach with unused value
===file===
<?php
if (rand(0, 1) > 0) {
    foreach ([1, 2, 3] as $val) {}
}

===expect===
UnusedForeachValue@3:26-3:30: Foreach value $val is never read
