===description===
A static method provided by a trait lands in the trait.
===cursor===
definition
===file===
<?php
trait Makes {
    public static function make(): static { return new static(); }
}
final class Widget { use Makes; }
Widget::ma<CURSOR>ke();
===expect===
test.php@3:4-3:66
