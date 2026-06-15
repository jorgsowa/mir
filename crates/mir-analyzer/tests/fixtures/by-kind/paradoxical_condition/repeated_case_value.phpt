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
ParadoxicalCondition@7:9-7:10: Value 0 is duplicated; this branch can never be reached
