===description===
Optional parameters before required ones, as PHP 8.4 reports them.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
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
//             ^^^^^^^^^^ OptionalParameterBeforeRequired: App\plain(): Optional parameter $a declared before required parameter $b is implicitly treated as a required parameter
function nullable(?int $a = null, $b) {}
//                ^^^^^^^^^^^^^^ OptionalParameterBeforeRequired: App\nullable(): Optional parameter $a declared before required parameter $b is implicitly treated as a required parameter
function implicit(int $a = null, $b) {}
//                ^^^^^^^^^^^^^ ImplicitlyNullableParameter: App\implicit(): Implicitly marking parameter $a as nullable is deprecated, the explicit nullable type must be used instead
function union(int|null $a = null, $b) {}
//             ^^^^^^^^^^^^^^^^^^ OptionalParameterBeforeRequired: App\union(): Optional parameter $a declared before required parameter $b is implicitly treated as a required parameter
function untyped($a = null, $b) {}
//               ^^^^^^^^^ OptionalParameterBeforeRequired: App\untyped(): Optional parameter $a declared before required parameter $b is implicitly treated as a required parameter
function variadic($a = 1, ...$rest) {}
function several($a = 1, $b = 2, $c, $d = 3, $e) {}
//               ^^^^^^ OptionalParameterBeforeRequired: App\several(): Optional parameter $a declared before required parameter $e is implicitly treated as a required parameter
//                       ^^^^^^ OptionalParameterBeforeRequired: App\several(): Optional parameter $b declared before required parameter $e is implicitly treated as a required parameter
//                                   ^^^^^^ OptionalParameterBeforeRequired: App\several(): Optional parameter $d declared before required parameter $e is implicitly treated as a required parameter

class Service {
    public function __construct(public int $retries = 3, $client) {}
//                              ^^^^^^^^^^^^^^^^^^^^^^^ OptionalParameterBeforeRequired: App\Service::__construct(): Optional parameter $retries declared before required parameter $client is implicitly treated as a required parameter
}
