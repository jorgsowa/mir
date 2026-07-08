===description===
FN: param-side checks (narrowing, param count, byref) only compared against
`all_parent_methods.first()` (the "primary" parent) instead of every
ancestor, unlike the return-type check which already loops all of them —
so a contravariance violation against a NON-primary ancestor interface was
silently missed depending on declaration order.
===config===
suppress=UnusedParam
===file===
<?php
class Animal {}
class Dog extends Animal {}
interface IB { public function f(Dog $a): void; }
interface IA { public function f(Animal $a): void; }
class C implements IB, IA {
    public function f(Dog $a): void {}
}
===expect===
MethodSignatureMismatch@7:4-7:38: Method C::f() signature mismatch: parameter $a type 'Dog' is narrower than parent type 'Animal'
