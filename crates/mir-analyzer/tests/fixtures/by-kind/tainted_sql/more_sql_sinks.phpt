===description===
Calls that run the SQL text they receive, as opposed to the parameters bound to it.
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <UndefinedClass errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Doctrine\DBAL {
    class Connection {
        public function executeQuery(string $sql, array $params = []): void {}
//                                   ^^^^^^^^^^^ UnusedParam: Parameter $sql is never used
//                                                ^^^^^^^^^^^^^^^^^^ UnusedParam: Parameter $params is never used
        public function executeStatement(string $sql, array $params = []): void {}
//                                       ^^^^^^^^^^^ UnusedParam: Parameter $sql is never used
//                                                    ^^^^^^^^^^^^^^^^^^ UnusedParam: Parameter $params is never used
        public function fetchOne(string $sql, array $params = []): void {}
//                               ^^^^^^^^^^^ UnusedParam: Parameter $sql is never used
//                                            ^^^^^^^^^^^^^^^^^^ UnusedParam: Parameter $params is never used
    }
}

namespace {
    function procedural($db): void {
        mysqli_execute_query($db, $_GET['q']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
        sqlsrv_query($db, $_GET['q']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function sqlsrv_query() is not defined
        sqlsrv_prepare($db, $_GET['q']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function sqlsrv_prepare() is not defined
    }

    function methods(mysqli $db, SQLite3 $lite, mysqli_stmt $stmt): void {
        $db->execute_query($_GET['q']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
        $lite->querySingle($_GET['q']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
        $stmt->prepare($_GET['q']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
    }

    function doctrine(Doctrine\DBAL\Connection $c): void {
        $c->executeQuery($_GET['q']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
        $c->executeStatement($_GET['q']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
        $c->fetchOne($_GET['q']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
    }

    function boundParameters(Doctrine\DBAL\Connection $c, mysqli $db): void {
        $c->executeQuery('SELECT ?', [$_GET['x']]);
        $db->execute_query('SELECT ?', [$_GET['x']]);
        mysqli_execute_query($db, 'SELECT ?', [$_GET['x']]);
        sqlsrv_query($db, 'SELECT ?', [$_GET['x']]);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function sqlsrv_query() is not defined
    }
}
