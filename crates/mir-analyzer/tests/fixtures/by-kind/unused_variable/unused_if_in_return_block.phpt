===description===
(divergence from Psalm: Psalm proves the foreach always returns on the
first iteration via const-array evaluation; mir keeps the no-return path,
on which `if ($i)` reads the line-2 write)
Unused if in return block
===file===
<?php
$i = rand(0, 1);

foreach ([1, 2, 3] as $a) {
    if ($a % 2) {
        $i = 7;
        return;
    }
}

if ($i) {}
===expect===
