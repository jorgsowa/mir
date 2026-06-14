===description===
Calling an @if-this-is method on $this inside the generic class body is not flagged
===config===
suppress=MissingPropertyType,UnusedParam,UnusedVariable
===file===
<?php
/** @template T */
class Box {
    /** @var T */ private $v;
    /** @param T $v */
    public function __construct($v) { $this->v = $v; }
    /** @if-this-is Box<int> */
    public function onlyInt(): void {}
    public function relay(): void {
        $this->onlyInt();
    }
}
$b = new Box(5);
$b->onlyInt();
===expect===
