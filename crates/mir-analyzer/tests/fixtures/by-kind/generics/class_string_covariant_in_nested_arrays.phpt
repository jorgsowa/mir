===description===
class-string<Child> is accepted where class-string<Parent> is expected, nested in lists, maps, shapes, unions and iterables
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class P {}
class C extends P {}
interface I {}
final class Impl implements I {}

/** @param class-string<P> $c */
function top(string $c): void {}
/** @param array{k: class-string<P>} $c */
function shape(array $c): void {}
/** @param array{k: array{j: class-string<P>}} $c */
function deepShape(array $c): void {}
/** @param list<class-string<P>> $c */
function lst(array $c): void {}
/** @param array<string, class-string<P>> $c */
function map(array $c): void {}
/** @param class-string<P>|null $c */
function nullable(?string $c): void {}
/** @param iterable<class-string<P>> $c */
function iter(iterable $c): void {}
/** @param array{k: class-string<I>} $c */
function ifaceShape(array $c): void {}

/**
 * @param class-string<C> $x
 * @param array{k: class-string<C>} $s
 * @param list<class-string<C>> $l
 * @param array<string, class-string<C>> $m
 * @param class-string<Impl> $impl
 * @param array{k: class-string<Impl>} $implShape
 */
function ok($x, $s, $l, $m, $impl, $implShape): void {
    top($x);
    shape($s);
    deepShape(['k' => $s]);
    lst($l);
    map($m);
    nullable($x);
    iter($l);
    shape(['k' => C::class]);
    lst([C::class]);
    ifaceShape($implShape);
    ifaceShape(['k' => Impl::class]);
}
