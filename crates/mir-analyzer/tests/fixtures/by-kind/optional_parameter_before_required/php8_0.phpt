===description===
Optional parameters before required ones, as PHP 8.0 reports them.
===config===
<mir>
  <phpVersion>8.0</phpVersion>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

function plain(int $a = 1, $b) {}
//                         ^^ OptionalParameterBeforeRequired: Required parameter $b follows optional parameter $a
function nullable(?int $a = null, $b) {}
function implicit(int $a = null, $b) {}
function union(int|null $a = null, $b) {}
function untyped($a = null, $b) {}
//                          ^^ OptionalParameterBeforeRequired: Required parameter $b follows optional parameter $a
function variadic($a = 1, ...$rest) {}
function several($a = 1, $b = 2, $c, $d = 3, $e) {}
//                               ^^ OptionalParameterBeforeRequired: Required parameter $c follows optional parameter $a
//                                           ^^ OptionalParameterBeforeRequired: Required parameter $e follows optional parameter $a

class Service {
    public function __construct(public int $retries = 3, $client) {}
//                                                       ^^^^^^^ OptionalParameterBeforeRequired: Required parameter $client follows optional parameter $retries
}

$closure = function ($a = 1, $b) { return $a . $b; };
//                           ^^ OptionalParameterBeforeRequired: Required parameter $b follows optional parameter $a
