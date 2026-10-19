===description===
mysqli_stmt::__construct() takes an optional ?string $query, so the statement can be
built from the connection alone, by subclasses and with named arguments.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

function build(mysqli $db, ?string $sql): void {
    $stmt = new mysqli_stmt($db);
    /** @mir-check $stmt is mysqli_stmt */
    $withQuery = new mysqli_stmt($db, 'SELECT 1');
    $nullQuery = new mysqli_stmt($db, null);
    $nullable = new mysqli_stmt($db, $sql);
    $named = new mysqli_stmt(mysql: $db);
    $reordered = new mysqli_stmt(query: 'SELECT 1', mysql: $db);
    $stmt->prepare('SELECT 1');
}

final class Statement extends mysqli_stmt {
    public function __construct(mysqli $db) {
        parent::__construct($db);
    }
}

class InheritedStatement extends mysqli_stmt {}

function subclasses(mysqli $db): void {
    $statement = new Statement($db);
    $inherited = new InheritedStatement($db);
    $inheritedQuery = new InheritedStatement($db, 'SELECT 1');
}

function misuse(mysqli $db): void {
    $none = new mysqli_stmt();
//          ^^^^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for mysqli_stmt::__construct(): expected 1, got 0
    $missingConnection = new mysqli_stmt(query: 'SELECT 1');
//                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for mysqli_stmt::__construct(): expected 1, got 0
    $surplus = new mysqli_stmt($db, 'SELECT 1', 1);
//                                              ^ TooManyArguments: Too many arguments for mysqli_stmt::__construct(): expected 2, got 3
    $inherited = new InheritedStatement();
//               ^^^^^^^^^^^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for InheritedStatement::__construct(): expected 1, got 0
    $badQuery = new mysqli_stmt($db, 1);
//                                   ^ InvalidArgument: Argument $query of mysqli_stmt::__construct() expects 'string|null', got '1'
    $badConnection = new mysqli_stmt('db');
//                                   ^^^^ InvalidArgument: Argument $mysql of mysqli_stmt::__construct() expects 'mysqli', got '"db"'
}
