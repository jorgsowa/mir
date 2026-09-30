===description===
`: static` and a trait's `: self` still bind to the receiver class.
===config===
suppress=UnusedMethod
===file===
<?php
class Base {
    public function returnsStatic(): static { return $this; }
}

trait Chain {
    public function chain(): self { return $this; }
}

class Sub extends Base {
    use Chain;

    public function subOnly(): void {}
}

(new Sub())->returnsStatic()->subOnly();
(new Sub())->chain()->subOnly();
(new Sub())->returnsStatic()->nope();
(new Sub())->chain()->nope();
===expect===
UndefinedMethod@18:0-18:36: Method Sub::nope() does not exist
UndefinedMethod@19:0-19:28: Method Sub::nope() does not exist
