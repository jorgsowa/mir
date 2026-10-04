===description===
class-string<Child> inside a shape nested in a map, list or shape satisfies class-string<Parent> in the declared return type
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class P {}
final class C extends P {}
interface I {}
final class Impl implements I {}

/** @return array<string, array{P, class-string<P>}> */
function mapOfShapes(): array {
    $r = ['a' => [new C(), C::class]];
    /** @mir-check $r is array{'a': array{0: C, 1: class-string<C>}} */
    return $r;
}

/** @return list<array{P, class-string<P>}> */
function listOfShapes(): array {
    return [[new C(), C::class]];
}

/** @return array{k: array{j: class-string<P>}} */
function shapeOfShapes(): array {
    return ['k' => ['j' => C::class]];
}

/** @return array{k: array{j: array{i: class-string<P>}}} */
function threeDeep(): array {
    return ['k' => ['j' => ['i' => C::class]]];
}

/** @return array<string, array{I, class-string<I>}> */
function interfaceBound(): array {
    return ['a' => [new Impl(), Impl::class]];
}

/** @return array<string, array{a: class-string<P>, b?: class-string<P>}> */
function optionalKey(): array {
    return ['x' => ['a' => C::class]];
}

/** @return array<string, array{class-string<P>}|null> */
function nullableShape(): array {
    return ['x' => [C::class], 'y' => null];
}

/** @return array<string, array{class-string<P>}> */
function viaVariable(): array {
    $c = C::class;
    return ['x' => [$c]];
}

/** @return array<string, array{0: array<string, class-string<P>>}> */
function shapeHoldingMap(): array {
    return ['x' => [['k' => C::class]]];
}
===expect===
