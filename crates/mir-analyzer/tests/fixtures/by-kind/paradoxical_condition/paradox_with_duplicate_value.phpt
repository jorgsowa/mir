===description===
Paradox with duplicate value
===file===
<?php
function foo(int $i) : void {
    echo match ($i) {
        1 => 0,
        1 => 1,
    };
};
===expect===
ParadoxicalCondition@5:8-5:9: Value 1 is duplicated; this branch can never be reached
