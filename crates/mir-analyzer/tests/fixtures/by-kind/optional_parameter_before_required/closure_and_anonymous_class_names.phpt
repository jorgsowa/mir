===description===
From PHP 8.4 a closure is named after its enclosing function, method or closure, and an anonymous class after its parent or first interface; PHP's NUL byte in an anonymous class name cuts a closure's name short.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
  <issueHandlers>
    <UndefinedClass errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedClosureParam errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

use Other\Base as B;

interface I {}

function outer(): void {
    $c = function ($a = 1, $b) {};
//                 ^^^^^^ OptionalParameterBeforeRequired: {closure:App\outer():9}(): Optional parameter $a declared before required parameter $b is implicitly treated as a required parameter
    $f = fn ($a = 1, $b) => 1;
//           ^^^^^^ OptionalParameterBeforeRequired: {closure:App\outer():11}(): Optional parameter $a declared before required parameter $b is implicitly treated as a required parameter
    $n = function () { return function ($a = 1, $b) {}; };
//                                      ^^^^^^ OptionalParameterBeforeRequired: {closure:{closure:App\outer():13}:13}(): Optional parameter $a declared before required parameter $b is implicitly treated as a required parameter
}

class K {
    public function m(): void { $c = function ($a = 1, $b) {}; }
//                                             ^^^^^^ OptionalParameterBeforeRequired: {closure:App\K::m():18}(): Optional parameter $a declared before required parameter $b is implicitly treated as a required parameter
}

$x = new class extends B { function m($a = 1, $b) {} };
//                                    ^^^^^^ OptionalParameterBeforeRequired: Other\Base@anonymous(): Optional parameter $a declared before required parameter $b is implicitly treated as a required parameter
$y = new class implements I { function m($a = 1, $b) {} };
//                                       ^^^^^^ OptionalParameterBeforeRequired: App\I@anonymous(): Optional parameter $a declared before required parameter $b is implicitly treated as a required parameter
$z = new class { function n() { $c = function ($a = 1, $b) {}; } };
//                                             ^^^^^^ OptionalParameterBeforeRequired: {closure:class@anonymous(): Optional parameter $a declared before required parameter $b is implicitly treated as a required parameter
