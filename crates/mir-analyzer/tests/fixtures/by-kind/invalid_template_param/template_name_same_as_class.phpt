===description===
Template param named "Type" where class "Type" appears inside another template's intersection bound — bound is re-evaluated per call site; a T that doesn't extend the bound-class is still caught
===file===
<?php
namespace App;

interface Named {}
class Type implements Named {}
class Both extends Type {}
class OnlyNamed implements Named {} // does NOT extend Type

/**
 * @template Type
 * @template T of Type&Named
 * @param Type $type_val
 * @param T $t
 */
function f($type_val, $t): void {
    $type_val;
    $t;
}

f(new Type(), new Both());      // Both extends Type & implements Named — OK
f(new Type(), new OnlyNamed()); // OnlyNamed does not extend Type — should error
===expect===
InvalidTemplateParam@21:1-21:31: Template type 'T' inferred as 'App\OnlyNamed' does not satisfy bound 'Type&App\Named'
