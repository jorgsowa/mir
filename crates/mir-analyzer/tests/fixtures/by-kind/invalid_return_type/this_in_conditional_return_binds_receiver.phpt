===description===
`$this` inside a conditional `@return` binds to the receiver like a plain `$this` return.
===file===
<?php
/** @template TParent */
class NodeDef {
    /** @param TParent $parent */
    public function __construct(public mixed $parent = null) {}
}

class ArrayNode {
    /**
     * @template T of 'array'|'scalar'
     * @param T $type
     * @return (T is 'array' ? NodeDef<$this> : NodeDef<null>)
     */
    public function prototype(string $type): NodeDef {
        return $type === 'array' ? new NodeDef($this) : new NodeDef();
    }

    /**
     * @return NodeDef<$this>
     */
    public function arrayPrototype(): NodeDef {
        return $this->prototype('array');
    }
}
