===description===
A class that extends a generic parent and implements a generic interface which
both name their template `T` resolves an inherited method against the
declaring ancestor's own type argument, not whichever ancestor bound `T` first.
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
class A {}
class B {}

/** @template T */
class Base {
    /** @return T */
    public function get() { throw new Exception(); }

    /** @param T $v */
    public function set($v): void {}
}

/** @template T */
interface Contract {
    /** @return T */
    public function fetch();
}

/**
 * @extends Base<A>
 * @implements Contract<B>
 */
class Impl extends Base implements Contract {
    public function fetch() { return new B(); }
}

/**
 * @extends Base<A>
 * @implements Contract<B>
 */
class ImplDoc extends Base implements Contract {
    /** {@inheritDoc} */
    public function get() { return new A(); }

    /** {@inheritDoc} */
    public function fetch() { return new B(); }
}

$i = new Impl();
$fromParent = $i->get();
/** @mir-check $fromParent is A */
$fromInterface = $i->fetch();
/** @mir-check $fromInterface is B */
$i->set(new A());
$i->set(new B());
//      ^^^^^^^ InvalidArgument: Argument $v of set() expects 'A', got 'B'

$d = new ImplDoc();
$docParent = $d->get();
/** @mir-check $docParent is A */
$docInterface = $d->fetch();
/** @mir-check $docInterface is B */
