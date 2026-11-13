===description===
An `instanceof` check on a property path deeper than `$a->b` narrows later reads
of the same path, until a write to any part of the path.
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
    <MixedPropertyFetch errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
abstract class Expr {}
final class Int_ extends Expr { public function __construct(public int $value) {} }
final class UnaryMinus extends Expr { public function __construct(public Expr $expr) {} }
final class Cond { public function __construct(public Expr $right) {} }
final class Where { public function __construct(public Cond $cond) {} }
final class Select { public function __construct(public Where $where) {} }

function twoHops(Cond $c): int {
    if ($c->right instanceof UnaryMinus && $c->right->expr instanceof Int_) {
        return -$c->right->expr->value;
    }
    return 0;
}

function threeHops(Select $s): int {
    if ($s->where->cond->right instanceof Int_) {
        return $s->where->cond->right->value;
    }
    return 0;
}

function negated(Select $s): int {
    if (!$s->where->cond->right instanceof Int_) {
        return 0;
    }
    return $s->where->cond->right->value;
}

function writeDropsNarrowing(Select $s, Cond $other): int {
    if ($s->where->cond->right instanceof Int_) {
        $s->where->cond = $other;
        return $s->where->cond->right->value;
//                                     ^^^^^ UndefinedProperty: Property Expr::$value does not exist
    }
    return 0;
}
