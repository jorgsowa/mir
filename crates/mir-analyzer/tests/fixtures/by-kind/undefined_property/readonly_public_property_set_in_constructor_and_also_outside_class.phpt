===description===
Readonly public property set in constructor and also outside class
===file===
<?php
class A {
    /**
     * @readonly-allow-private-mutation
     */
    public string $bar;

    public function __construct() {
        $this->bar = "hello";
    }

    public function setAgain() : void {
        $this->bar = "hello";
    }
}

$a = new A();
$a->bar = "goodbye";
===expect===
