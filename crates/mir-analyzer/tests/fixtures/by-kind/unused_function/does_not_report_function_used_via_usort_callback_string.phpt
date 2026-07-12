===description===
a function used only as a bare string callback to usort must not be reported unused
===config===
suppress=
===file===
<?php
function my_cmp(int $a, int $b): int { return $a <=> $b; }

$items = [3, 1, 2];
usort($items, 'my_cmp');
===expect===
