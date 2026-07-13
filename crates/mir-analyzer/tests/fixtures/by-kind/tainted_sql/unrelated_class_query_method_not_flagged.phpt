===description===
A query() method on a class unrelated to PDO/mysqli/SQLite3 is not treated as a SQL sink.
===config===
suppress=UnusedParam,MixedArrayAccess,MixedArgument
===file===
<?php
class SearchIndex {
    public function query(string $q): void {}
}
function run(SearchIndex $index): void {
    $index->query($_GET['q']);
}
===expect===
