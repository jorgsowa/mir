===description===
Normal new instantiation is not flagged (only ->__construct() is)
===file===
<?php
class A {
    public function __construct() {}
    public function greet(): string { return 'hello'; }
}
$a = new A;
echo $a->greet();
$b = new A();
echo $b->greet();
===expect===
