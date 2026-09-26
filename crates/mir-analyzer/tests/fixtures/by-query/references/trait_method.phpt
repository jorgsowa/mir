===description===
A trait method's references include calls through every using class.
===cursor===
references
===file===
<?php
trait Greets {
    public function greet(): string { return 'hi'; }
}
final class A { use Greets; }
final class B { use Greets; }
(new A())->gr<CURSOR>eet();
(new B())->greet();
===expect===
test.php@7:11-7:16
test.php@8:11-8:16
