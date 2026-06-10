===description===
Callable with spaces either side of colon bad var arg
===ignore===
TODO
===file===
<?php
class C {
    /**
     * @var callable(string, string) : bool $p
     */
    public $p;

    public function __construct() {
        $this->p = function (string $s, string $t): stdClass {
            return new stdClass;
        };
    }
}
===expect===
