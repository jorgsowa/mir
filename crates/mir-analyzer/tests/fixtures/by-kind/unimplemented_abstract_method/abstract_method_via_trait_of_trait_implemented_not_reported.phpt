===description===
The trait-of-trait abstract method above must not be flagged once the
composing class actually implements it.
===file===
<?php
trait Leaf {
    abstract public function foo(): void;
}
trait Mid {
    use Leaf;
}
class Complete {
    use Mid;

    public function foo(): void {}
}
===expect===
