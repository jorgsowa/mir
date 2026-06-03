===description===
Loop set if null with break without reference
===file===
<?php
$a = null;

while (rand(0, 1)) {
    if ($a !== null) {
        $a = 4;
        break;
    }

    $a = 5;
}
===expect===
RedundantCondition@5:9-5:20: Condition is always true/false for type 'bool'
