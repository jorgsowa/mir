===description===
method call
===file===
<?php
class Foo {
    public function bar(int $n): void { var_dump($n); }
}

$f = new Foo();
$f->bar(null);
===expect===
NullArgument@7:8: Argument $n of bar() cannot be null
