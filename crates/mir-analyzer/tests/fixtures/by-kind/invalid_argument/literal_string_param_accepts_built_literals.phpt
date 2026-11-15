===description===
Strings built only from literals stay `literal-string`: interpolation of literal
parts, a query extended with `.=` in a loop, long concatenations, magic constants,
and `implode()`, `str_repeat()`, `rtrim()` and `sprintf()` of literal arguments.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedForeachValue errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param literal-string $sql */
function query(string $sql): void {}

/**
 * @param list<int> $ids
 * @param literal-string $table
 */
function run(array $ids, string $table, bool $desc): void {
    query("SELECT * FROM $table");
    query(<<<SQL
    SELECT * FROM {$table} WHERE id = 1
    SQL);
    $sql = 'SELECT * FROM t WHERE 1 = 1';
    foreach ($ids as $id) {
        $sql .= ' AND x = ?';
    }
    query($sql);
    query(str_repeat('a', 600) . str_repeat('b', 600));
    query(str_repeat('x', 600) . 'y' . str_repeat('z', 600));
    query('SELECT * FROM ' . __CLASS__);
    query(implode(', ', ['id', 'name']));
    query(rtrim(str_repeat('?,', count($ids)), ','));
    query(sprintf('SELECT * FROM %s LIMIT %d', $table, 10));
    query('ORDER BY id ' . ($desc ? 'DESC' : 'ASC'));
}
