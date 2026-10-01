===description===
A method called through an intersection-typed property resolves on the member that declares it.
===cursor===
definition
===file===
<?php
interface Mock {}
class Svc {
    public function getTotal(): int { return 1; }
}
class Test {
    /** @var Mock&Svc */
    private $sut;
    public function run(): int {
        return $this->sut->getTo<CURSOR>tal();
    }
}
===expect===
test.php@4:4-4:49
