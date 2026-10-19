===description===
Before PHP 8.0 the mysqli_stmt constructor is untyped, and $query is still optional.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>7.4</phpVersion>
</mir>
===file===
<?php

function build(mysqli $db): void {
    $stmt = new mysqli_stmt($db);
    /** @mir-check $stmt is mysqli_stmt */
    $withQuery = new mysqli_stmt($db, 'SELECT 1');
    $none = new mysqli_stmt();
//          ^^^^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for mysqli_stmt::__construct(): expected 1, got 0
}
