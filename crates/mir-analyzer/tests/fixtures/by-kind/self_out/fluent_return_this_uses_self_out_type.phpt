===description===
A fluent `@return $this` method returns the receiver as its `@psalm-this-out` leaves it
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface ParentNode {}
final class Root implements ParentNode {}

/** @template TParent of ParentNode|null = null */
class Node {
    /**
     * @template TNew of ParentNode|null
     * @param TNew $parent
     * @psalm-this-out static<TNew>
     * @return $this
     */
    public function setParent(?ParentNode $parent): static {
        return $this;
    }

    /** @return TParent */
    public function end(): ?ParentNode {
        return null;
    }
}

/** @param Node<null> $node */
function f(Node $node, Root $root): ParentNode {
    $withParent = $node->setParent($root);
    /** @mir-check $withParent is Node<Root> */
    return $withParent->end();
}
