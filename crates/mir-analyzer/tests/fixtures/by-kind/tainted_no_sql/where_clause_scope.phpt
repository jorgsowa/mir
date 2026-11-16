===description===
A tainted `$where` clause is reported in a query filter, also when the filter is
built in a variable first or nests it under `$or`; an array that never reaches a
query, and a literal `$where`, aren't.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <TaintedHtml errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace MongoDB {
    class Collection {
        public function find(array|object $filter = [], array $options = []): \Iterator { return new \ArrayIterator([]); }
    }
}

namespace {
    function notAQuery(): void {
        $config = ['$where' => $_GET['note']];
        echo json_encode($config);
    }

    function builtFirst(\MongoDB\Collection $users): void {
        $filter = ['$where' => $_GET['w']];
        $users->find($filter);
//                   ^^^^^^^ TaintedNoSql: Tainted NoSQL query — possible NoSQL injection
    }

    function nestedUnderOr(\MongoDB\Collection $users): void {
        $users->find(['$or' => [['$where' => $_GET['w']], ['active' => true]]]);
//                                           ^^^^^^^^^^ TaintedNoSql: Tainted NoSQL query — possible NoSQL injection
    }

    function literalClause(\MongoDB\Collection $users): void {
        $users->find(['$where' => 'this.score > 10']);
    }
}
