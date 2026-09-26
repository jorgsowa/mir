===description===
Go-to-definition on a method provided by a trait lands in the trait.
===cursor===
definition
===file===
<?php
trait Greets {
    public function greet(): string { return 'hi'; }
}
final class Greeter { use Greets; }
echo (new Greeter())->gr<CURSOR>eet();
===expect===
test.php@3:4-3:52
