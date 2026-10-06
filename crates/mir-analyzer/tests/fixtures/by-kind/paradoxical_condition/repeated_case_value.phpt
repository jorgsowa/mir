===description===
Repeated case value
===file===
<?php
$a = rand(0, 1);
switch ($a) {
    case 0:
        break;

    case 0:
//       ^ ParadoxicalCondition: Value 0 is duplicated; this branch can never be reached
        echo "I never get here";
}
