===description===
An override with only a native return hint inherits the parent's docblock refinement
(`class-string<T>`, `positive-int`); the wider native hint is not a mismatch.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Item {}

/** @template T */
abstract class C {
    /** @return class-string<T> */
    abstract public static function k(): string;
}

/** @template-extends C<Item> */
final class D extends C {
    public static function k(): string {
        return Item::class;
    }
}

/** @template T of int */
interface HasCount {
    /** @return T */
    public function count(): int;
}

/** @template-implements HasCount<positive-int> */
final class Three implements HasCount {
    public function count(): int {
        return 3;
    }
}

$k = D::k();
/** @mir-check $k is string */
$n = (new Three())->count();
/** @mir-check $n is int */
===expect===
