===description===
reports object or false to object param
===file===
<?php
class Connection {}
function takesConnection(Connection $c): void { var_dump($c); }
/** @return Connection|false */
function getConnection(): Connection|false { return new Connection(); }
function test(): void {
    takesConnection(getConnection());
}
===expect===
PossiblyInvalidArgument@7:21: Argument $c of takesConnection() expects 'Connection', possibly different type 'Connection|false' provided
