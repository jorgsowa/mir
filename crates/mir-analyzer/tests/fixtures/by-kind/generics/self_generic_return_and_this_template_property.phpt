===description===
`@return self<$this>` keeps its type argument, so a fluent builder's `end()`
returns the right level, and a template-typed property read or written through
`$this` stays the open template instead of taking the template's default.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface ParentNode {}

/** @template TParent of ParentNode|null = null */
class NodeDef implements ParentNode {
    /** @var TParent */
    protected ?ParentNode $parent = null;

    public function setParent(?ParentNode $parent): static {
        $this->parent = $parent;
        return $this;
    }

    /** @param TParent $parent */
    public function setTypedParent($parent): void {
        $this->parent = $parent;
    }

    /** @return TParent */
    public function end(): ?ParentNode {
        $parent = $this->parent;
        return $parent;
    }

    public function info(string $s): static { return $this; }

    /** @param NodeDef $other */
    public function copyTo(NodeDef $other, ?ParentNode $parent): void {
        $other->parent = $parent;
//      ^^^^^^^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $parent expects 'null', cannot assign 'ParentNode|null'
    }
}

/**
 * @template TParent of ParentNode|null = null
 * @extends NodeDef<TParent>
 */
class StringDef extends NodeDef {}

/**
 * @template TParent of ParentNode|null = null
 * @extends NodeDef<TParent>
 */
class ArrayDef extends NodeDef {
    /** @return StringDef<$this> */
    public function stringPrototype(): StringDef { return new StringDef(); }

    /** @return self<$this> */
    public function arrayPrototype(): self { return new self(); }

    /** @return Builder<static> */
    public function children(): Builder { return new Builder(); }
}

/** @template TParent of ParentNode|null = null */
class Builder implements ParentNode {
    /** @return ArrayDef<$this> */
    public function arrayNode(string $name): ArrayDef { return new ArrayDef(); }

    /** @return TParent */
    public function end(): ?ParentNode { return null; }
}

function build(Builder $root): void {
    $proto = $root->arrayNode('a')->arrayPrototype();
    /** @trace $proto */
    echo 1;
//  ^^^^^^^ Trace: Type of $proto is ArrayDef<ArrayDef<Builder>>
    $up = $proto->end();
    /** @trace $up */
    echo 1;
//  ^^^^^^^ Trace: Type of $up is ArrayDef<Builder>
    $children = $proto->children();
    /** @trace $children */
    echo 1;
//  ^^^^^^^ Trace: Type of $children is Builder<ArrayDef<ArrayDef<Builder>>>

    $root
        ->arrayNode('codes')
            ->arrayPrototype()
                ->children()
                    ->arrayNode('methods')
                        ->stringPrototype()->end()
                        ->info('x')
                    ->end()
                ->end()
            ->info('y')
        ->end()
        ->info('z');
}
