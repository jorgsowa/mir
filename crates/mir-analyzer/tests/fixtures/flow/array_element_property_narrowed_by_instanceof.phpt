===description===
An `instanceof` check on a property read through a literal array index
(`$args[1]->value`, `$m->stmts[0]->expr`) narrows later reads of the same
path, until the array or that element is written.
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
final class Name { public function __construct(public string $s) {} }
final class FuncCall extends Expr { public function __construct(public Name $name) {} }
final class Variable extends Expr { public string $name = ''; }
final class PropertyFetch extends Expr { public function __construct(public Expr $var) {} }
final class Arg { public function __construct(public Expr $value) {} }
abstract class Stmt {}
final class Return_ extends Stmt { public ?Expr $expr = null; }
final class Method { /** @var list<Stmt> */ public array $stmts = []; }

/** @param list<Arg> $args */
function localList(array $args): string {
    if (isset($args[1]) && $args[1]->value instanceof FuncCall) {
        return $args[1]->value->name->s;
    }
    return '';
}

/** @param array{opts: Arg} $named */
function stringKey(array $named): string {
    if ($named['opts']->value instanceof FuncCall) {
        return $named['opts']->value->name->s;
    }
    return '';
}

function propertyList(Method $m): bool {
    return $m->stmts
        && $m->stmts[0] instanceof Return_
        && $m->stmts[0]->expr instanceof PropertyFetch
        && $m->stmts[0]->expr->var instanceof Variable
        && $m->stmts[0]->expr->var->name === 'this';
}

/** @param list<Arg> $args */
function elementWrite(array $args, Arg $other): string {
    if (isset($args[1]) && $args[1]->value instanceof FuncCall) {
        $args[1] = $other;
        return $args[1]->value->name->s;
//                              ^^^^ UndefinedProperty: Property Expr::$name does not exist
    }
    return '';
}

/** @param list<Arg> $args */
function reassigned(array $args, Arg $other): string {
    if (isset($args[0]) && $args[0]->value instanceof FuncCall) {
        $args = [$other];
        return $args[0]->value->name->s;
//                              ^^^^ UndefinedProperty: Property Expr::$name does not exist
    }
    return '';
}
