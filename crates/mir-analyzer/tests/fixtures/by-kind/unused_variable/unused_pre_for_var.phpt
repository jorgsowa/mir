===description===
Unused pre for var
===file===
<?php
$i = 0;

for ($i = 0; $i < 10; $i++) {
    echo $i;
}
===expect===
UnusedVariable
===ignore===
TODO
