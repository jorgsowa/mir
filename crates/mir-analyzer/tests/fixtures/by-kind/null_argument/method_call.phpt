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
NullArgument@7:9-7:13: Argument $n of bar() cannot be null
