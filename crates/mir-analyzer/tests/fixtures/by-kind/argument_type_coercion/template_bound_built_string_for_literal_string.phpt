===description===
A string built from runtime parts, alone or beside a literal, is a coercion to a `literal-string|int|float` template bound.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T of literal-string|int|float */
final class Raw {
    /** @param T $value */
    public function __construct(public $value) {}
}

function build(string $column, bool $now): void {
    new Raw("count($column)");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $value of Raw::__construct() expects 'literal-string|int|float', got 'non-empty-string' — coercion may fail at runtime
    new Raw($now ? 'CURRENT_TIMESTAMP' : "'$column'");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $value of Raw::__construct() expects 'literal-string|int|float', got '"CURRENT_TIMESTAMP"|non-empty-string' — coercion may fail at runtime
}
