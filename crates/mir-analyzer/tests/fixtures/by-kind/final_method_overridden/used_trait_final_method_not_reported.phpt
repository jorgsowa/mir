===description===
FP: a class that merely `use`s a trait declaring a `final` method — never
redeclaring it in its own body — was flagged as overriding that final method.
Flattening a trait's method into a class isn't an override (verified live:
this loads with no fatal).
===file===
<?php
trait Greeter {
    final protected function greet(): string { return 'hi'; }
}
class Widget {
    use Greeter;
}
===expect===
