===description===
reports object or false to object param
===config===
suppress=ForbiddenCode
===file===
<?php
class Connection {}
function takesConnection(Connection $c): void { var_dump($c); }
/** @return Connection|false */
function getConnection(): Connection|false { return new Connection(); }
function test(): void {
    takesConnection(getConnection());
//                  ^^^^^^^^^^^^^^^ PossiblyInvalidArgument: Argument $c of takesConnection() expects 'Connection', possibly different type 'Connection|false' provided
}
===expect===
