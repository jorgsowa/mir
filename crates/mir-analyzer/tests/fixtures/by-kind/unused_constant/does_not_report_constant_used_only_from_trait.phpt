===description===
A trait body's `self::X` reads the composing class's constant
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
trait ReadsLimit {
    public function limit(): int {
        return self::LIMIT;
//      ^^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
    }
}

final class Paginator {
    use ReadsLimit;

    private const LIMIT = 10;
}

(new Paginator())->limit();
