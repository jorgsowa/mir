===description===
`self` / `static` used as a type argument of `@template-implements`,
`@template-extends` and `@template-use` resolves to the declaring class,
so an override returning `self` is compatible with the bound template.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
interface Cloner {
    /** @return T */
    public function copy(): object;
}

/** @template-implements Cloner<self> */
final class Order implements Cloner {
    public function copy(): self { return new self(); }
}

/** @template T */
abstract class Base {
    /** @return T */
    abstract public function make(): object;
}

/** @template-extends Base<self> */
final class Widget extends Base {
    public function make(): self { return new self(); }
}

/** @template T */
interface Source {
    /** @return T */
    public function fetch(): object;
}

/** @template-extends Source<self> */
interface Node extends Source {}

final class Leaf implements Node {
    public function fetch(): self { return new self(); }
}

/** @template-implements Cloner<self> */
enum Suit implements Cloner {
    case Hearts;
    public function copy(): self { return self::Hearts; }
}

/**
 * @template T
 * @param Cloner<T> $c
 * @return T
 */
function cloneOf(Cloner $c) {
    return $c->copy();
}

$o = cloneOf(new Order());
/** @mir-check $o is self(Order) */
echo "ok";
