===description===
Under @psalm-immutable/@psalm-external-mutation-free, passing a reachable
object to a side-effect-free builtin is not a mutation; an impure builtin
still is.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Node {}

/** @psalm-immutable */
final class Tree {
    public Node|Closure $root;
    /** @var ArrayIterator<int, int> */
    public ArrayIterator $items;

    public function check(): bool {
        assert($this->root instanceof Node);
        return is_callable($this->root);
    }

    /** @return array<int, int> */
    public function copy(): array {
        return iterator_to_array($this->items);
//             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function iterator_to_array() in a @pure function
    }
}

final class Visitor {
    /** @psalm-external-mutation-free */
    public function visit(Node $node): bool {
        return is_callable($node);
    }
}
