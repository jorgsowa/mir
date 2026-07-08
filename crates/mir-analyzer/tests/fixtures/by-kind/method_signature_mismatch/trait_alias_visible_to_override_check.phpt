===description===
FN: a child class overriding a name that only exists via a parent's trait
alias (`use T { orig as newName; }`) got no signature check at all — the
ancestor walk only looked at each ancestor's own_methods, which never
contains a name that only exists through an alias.
===config===
suppress=UnusedParam
===file===
<?php
trait T { public function greet(string $s): void {} }
class Base { use T { greet as sayHello; } }
class Child extends Base {
    public function sayHello(int $s): void {}
}
===expect===
MethodSignatureMismatch@5:4-5:45: Method Child::sayhello() signature mismatch: parameter $s type 'int' is narrower than parent type 'string'
