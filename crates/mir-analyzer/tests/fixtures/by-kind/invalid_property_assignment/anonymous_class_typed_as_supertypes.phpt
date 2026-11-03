===description===
`new class extends A implements I {}` is typed as its declared supertypes, so it
satisfies an `A`/`I`-typed property. One that declares public members its
supertypes don't have, or none at all, stays `object`.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Greeter { public function greet(): string; }
interface Named { public function name(): string; }
abstract class Base { abstract public function run(): void; }
trait Greets { public function greet(): string { return ''; } }

final class Holder {
    private ?Greeter $greeter = null;
    private \ArrayAccess $access;
    private Base $base;
    private Greeter&Named $both;
    private ?Named $named = null;

    public function supertypes(): void {
        $this->greeter = new class implements Greeter {
            public function greet(): string { return ''; }
        };
        $this->greeter = new class implements Greeter { use Greets; };
        $this->access = new class extends \ArrayObject {};
        $this->base = new class extends Base {
            public function run(): void {}
        };
        $this->both = new class implements Greeter, Named {
            public function greet(): string { return ''; }
            public function name(): string { return ''; }
        };
        $base = new class extends Base {
            private int $runs = 0;
            public function run(): void { $this->runs++; }
        };
        /** @trace $base */
        echo 1;
//      ^^^^^^^ Trace: Type of $base is Base
    }

    public function extraMembersStayObject(): array {
        $o = new class implements Greeter {
            public function greet(): string { return ''; }
            public function extra(): int { return 1; }
        };
        $p = new class extends Base {
            public int $count = 0;
            public function run(): void {}
        };
        $q = new class(1) extends Base {
            public function __construct(public int $size) {}
            public function run(): void {}
        };
        return [$o->extra(), $p->count, $q->size];
    }

    public function wrongSupertype(): void {
        $this->named = new class extends \ArrayObject {};
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $named expects 'Named|null', cannot assign 'ArrayObject'
    }
}
