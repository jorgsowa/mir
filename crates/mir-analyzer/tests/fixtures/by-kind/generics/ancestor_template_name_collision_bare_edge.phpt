===description===
Two ancestors that both name a template `T`: a typed `@extends` on one must
not bind the other's `T` when it is only reached through a bare `implements`.
The inherited method is neither flagged as a signature mismatch nor typed as
the unrelated ancestor's argument.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Kind {}
class Item {}

/** @template T of object */
abstract class Holder {
    /** @param T $x */
    public function put($x): void {}
}

/** @template T of Kind */
interface HasKind {
    /** @return T */
    public function kind();
    /** @param T $k */
    public function take($k): void;
}

/** @extends Holder<Item> */
final class Impl extends Holder implements HasKind {
    public function kind(): Kind { return new Kind(); }
    public function take(Kind $k): void {}
}

function bareEdgeKeepsNativeReturn(Impl $o): void {
    $k = $o->kind();
    /** @mir-check $k is Kind */
    echo "ok";
}

function typedAncestorStillBinds(Impl $o): void {
    $o->put(new Item());
}

/** @implements HasKind<Kind> */
final class TypedImpl extends Holder implements HasKind {
    public function kind(): Kind { return new Kind(); }
    public function take(Kind $k): void {}
}

function typedEdgeBindsOwnArgs(TypedImpl $o): void {
    $k = $o->kind();
    /** @mir-check $k is Kind */
    echo "ok";
}
