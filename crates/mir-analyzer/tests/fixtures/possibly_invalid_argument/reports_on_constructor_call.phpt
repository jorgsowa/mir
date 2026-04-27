===file===
<?php
class Query {
    public function __construct(string $sql) { var_dump($sql); }
}
/** @return string|false */
function buildSql(): string|false { return 'SELECT 1'; }
function test(): void {
    new Query(buildSql());
}
===expect===
PossiblyInvalidArgument: Argument $sql of Query::__construct() expects 'string', possibly different type 'string|false' provided
