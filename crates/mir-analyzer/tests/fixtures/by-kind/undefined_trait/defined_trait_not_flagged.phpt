===description===
UndefinedTrait does NOT fire when the trait exists.
===file===
<?php
trait ExistingTrait {
    public function greet(): string { return "hello"; }
}

class Foo {
    use ExistingTrait;
}

===expect===
