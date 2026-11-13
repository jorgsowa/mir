===description===
A runtime string passed where a class template is bound by `literal-string|int` is a coercion on the constructor argument; literals satisfy the bound.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T of literal-string|int */
final class Expr {
    /** @param T $value */
    public function __construct(public $value) {}
}

function build(string $sql): void {
    new Expr('select 1');
    new Expr(5);
    new Expr($sql);
//  ^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $value of Expr::__construct() expects 'literal-string|int', got 'string' — coercion may fail at runtime
}
