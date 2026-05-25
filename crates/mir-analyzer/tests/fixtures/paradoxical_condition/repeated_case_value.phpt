===description===
Repeated case value
===file===
<?php
$a = rand(0, 1);
switch ($a) {
    case 0:
        break;

    case 0:
        echo "I never get here";
}
===expect===
ParadoxicalCondition
===ignore===
TODO
