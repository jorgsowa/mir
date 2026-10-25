===description===
Optional parameters before required ones, as PHP 8.2 reports them.
===config===
<mir>
  <phpVersion>8.2</phpVersion>
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
//             ^^^^^^^^^^ OptionalParameterBeforeRequired: Optional parameter $a declared before required parameter $b is implicitly treated as a required parameter
function nullable(?int $a = null, $b) {}
//                ^^^^^^^^^^^^^^ OptionalParameterBeforeRequired: Optional parameter $a declared before required parameter $b is implicitly treated as a required parameter
function implicit(int $a = null, $b) {}
function union(int|null $a = null, $b) {}
function untyped($a = null, $b) {}
//               ^^^^^^^^^ OptionalParameterBeforeRequired: Optional parameter $a declared before required parameter $b is implicitly treated as a required parameter
function variadic($a = 1, ...$rest) {}
function several($a = 1, $b = 2, $c, $d = 3, $e) {}
//               ^^^^^^ OptionalParameterBeforeRequired: Optional parameter $a declared before required parameter $e is implicitly treated as a required parameter
//                       ^^^^^^ OptionalParameterBeforeRequired: Optional parameter $b declared before required parameter $e is implicitly treated as a required parameter
//                                   ^^^^^^ OptionalParameterBeforeRequired: Optional parameter $d declared before required parameter $e is implicitly treated as a required parameter

class Service {
    public function __construct(public int $retries = 3, $client) {}
//                              ^^^^^^^^^^^^^^^^^^^^^^^ OptionalParameterBeforeRequired: Optional parameter $retries declared before required parameter $client is implicitly treated as a required parameter
}

$closure = function ($a = 1, $b) { return $a . $b; };
//                   ^^^^^^ OptionalParameterBeforeRequired: Optional parameter $a declared before required parameter $b is implicitly treated as a required parameter
