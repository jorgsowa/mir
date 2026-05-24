===description===
paradoxWithDuplicateValue
===file===
<?php
function foo(int $i) : void {
    echo match ($i) {
        1 => 0,
        1 => 1,
    };
};
===expect===
ParadoxicalCondition
===ignore===
TODO
