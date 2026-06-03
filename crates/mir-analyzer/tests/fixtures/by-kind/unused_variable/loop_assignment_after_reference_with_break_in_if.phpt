===description===
Loop assignment after reference with break in if
===ignore===
TODO: loop merge uses pre-loop state as the else-path, reintroducing the
$a=0 pending write even after echo $a consumed it. The dead write is $a=1
inside the if-break branch (line 7), not $a=0 (line 2). Needs per-iteration
last_write_locs tracking that doesn't bleed pre-loop state through the else-path.
===file===
<?php
$a = 0;
while (rand(0, 1)) {
    echo $a;

    if (rand(0, 1)) {
        $a = 1;
        break;
    }
}
===expect===
UnusedVariable
