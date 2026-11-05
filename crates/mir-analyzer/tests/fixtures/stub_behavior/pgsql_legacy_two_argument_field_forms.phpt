===file===
<?php
function f(\PgSql\Result $r): void {
    pg_fetch_result($r, 'a');
    pg_field_is_null($r, 'a');
    pg_field_prtlen($r, 0);
    pg_field_is_null($r, 0, 'a');
}
