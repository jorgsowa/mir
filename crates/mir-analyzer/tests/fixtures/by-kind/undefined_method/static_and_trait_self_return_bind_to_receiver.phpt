===description===
`: static` and a trait's `: self` still bind to the receiver class.
===config===
<mir>
  <issueHandlers>
    <UnusedMethod errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Sub::nope() does not exist
(new Sub())->chain()->nope();
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Sub::nope() does not exist
===expect===
