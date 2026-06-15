===description===
Unused pre for var
===file===
<?php
$i = 0;

for ($i = 0; $i < 10; $i++) {
    echo $i;
}
===expect===
UnusedVariable@2:0-2:2: Variable $i is never read
