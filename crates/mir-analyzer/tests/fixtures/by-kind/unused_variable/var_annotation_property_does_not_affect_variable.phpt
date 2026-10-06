===description===
@var T $obj->prop leaves $obj itself untouched, and plain @var $x still narrows the variable
===file===
<?php
class Box {
    public mixed $prop = null;
}
function p(Box $obj, mixed $x): void {
    /** @var int $obj->prop */
    $obj->prop = 1;
    /** @mir-check $obj is Box */
    echo $obj->prop;
    /** @var string $x */
    $x = strtolower('A');
    /** @mir-check $x is string */
    echo $x;
}
