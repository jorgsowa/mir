===description===
Two directly-used traits both declare a `final` copy of the same method name,
resolved via `insteadof`; the losing trait's `final` copy must not make the
composing class look like it's overriding anything (verified live: no fatal).
===file===
<?php
trait A { final public function greet(): string { return 'A'; } }
trait B { final public function greet(): string { return 'B'; } }
class Widget {
    use A, B { A::greet insteadof B; }
}
===expect===
