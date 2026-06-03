===description===
Unused pre for var
===file===
<?php
$i = 0;

for ($i = 0; $i < 10; $i++) {
    echo $i;
}
===expect===
UnusedVariable@2:1-2:3: Variable $i is never read
