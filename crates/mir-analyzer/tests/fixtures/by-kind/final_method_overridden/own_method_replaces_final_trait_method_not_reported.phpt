===description===
FP: a class's own method with the same name as a `final` method from a
directly-used trait was flagged as an override. A class member always takes
precedence over a same-named trait method at composition time — verified
live: this loads with no fatal, and calling greet() dispatches to the
class's own copy, not the trait's.
===file===
<?php
trait Greeter {
    final protected function greet(): string { return 'hi'; }
}
class Widget {
    use Greeter;
    protected function greet(): string { return 'bye'; }
}
===expect===
