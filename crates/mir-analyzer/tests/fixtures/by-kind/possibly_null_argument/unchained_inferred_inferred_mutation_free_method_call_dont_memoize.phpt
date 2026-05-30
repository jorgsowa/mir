===description===
Unchained inferred inferred mutation free method call dont memoize
===file===
<?php
class SomeClass {
    private ?int $int;

    public function __construct() {
        $this->int = 1;
    }

    public function getInt(): ?int {
        return $this->int;
    }
}

function printInt(int $int): void {
    echo $int;
}

$obj = new SomeClass();

if ($obj->getInt() !== null) {
    printInt($obj->getInt());
}
===expect===
PossiblyNullArgument@21:14-21:28: Argument $int of printInt() might be null
