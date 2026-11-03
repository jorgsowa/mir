===description===
`parent` in a called method's return type resolves to the declaring class's
parent (for a trait, the parent of the class using it), and `self`/`parent` in
an inline `@var` resolve to the enclosing class.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {}
class Lazy extends Base {
    public function native(): parent { return new Base(); }
    /** @return parent */
    public function doc() { return new Base(); }
    /** @return list<parent> */
    public function many(): array { return [new Base()]; }
    public function wrong(): parent { return new \stdClass(); }
//                                    ^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'stdClass' is not compatible with declared 'parent(Lazy)'
}
class LazierChild extends Lazy {}

trait ReturnsParent {
    public function viaTrait(): parent { return new Base(); }
}
class UsesTrait extends Base { use ReturnsParent; }
class UsesTraitChild extends UsesTrait {}

function calls(Lazy $l, LazierChild $c, UsesTrait $t, UsesTraitChild $tc): void {
    $a = $l->native();
    /** @trace $a */
    echo 1;
//  ^^^^^^^ Trace: Type of $a is Base
    $b = $l->doc();
    /** @trace $b */
    echo 1;
//  ^^^^^^^ Trace: Type of $b is Base
    $m = $l->many();
    /** @trace $m */
    echo 1;
//  ^^^^^^^ Trace: Type of $m is list<Base>
    $d = $c->native();
    /** @trace $d */
    echo 1;
//  ^^^^^^^ Trace: Type of $d is Base
    $e = $t->viaTrait();
    /** @trace $e */
    echo 1;
//  ^^^^^^^ Trace: Type of $e is Base
    $f = $tc->viaTrait();
    /** @trace $f */
    echo 1;
//  ^^^^^^^ Trace: Type of $f is Base
}

final class Items extends Base {
    private array $items = [];

    public function current(): self {
        /** @var self $x */
        $x = current($this->items);
        return $x;
    }

    public function maybe(): ?self {
        /** @var self|null $x */
        $x = current($this->items) ?: null;
        return $x;
    }

    public function up(): parent {
        /** @var parent $x */
        $x = current($this->items);
        return $x;
    }
}

trait SelfInTrait {
    public function me(): self {
        /** @var self $x */
        $x = $this;
        return $x;
    }
}
