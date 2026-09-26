===description===
An inherited method call reports the declaring class.
===cursor===
symbol
===file===
<?php
class Base {
    public function greet(): string { return 'hi'; }
}
final class Child extends Base {}
echo (new Child())->gr<CURSOR>eet();
===expect===
kind: method call Base::greet
type: string
