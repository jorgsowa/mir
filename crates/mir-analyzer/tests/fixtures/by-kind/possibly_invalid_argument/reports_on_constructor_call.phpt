===description===
reports on constructor call
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Query {
    public function __construct(string $sql) { var_dump($sql); }
}
/** @return string|false */
function buildSql(): string|false { return 'SELECT 1'; }
function test(): void {
    new Query(buildSql());
//            ^^^^^^^^^^ PossiblyInvalidArgument: Argument $sql of Query::__construct() expects 'string', possibly different type 'string|false' provided
}
