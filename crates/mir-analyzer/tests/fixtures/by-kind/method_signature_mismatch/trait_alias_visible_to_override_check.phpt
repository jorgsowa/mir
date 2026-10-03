===description===
FN: a child class overriding a name that only exists via a parent's trait
alias (`use T { orig as newName; }`) got no signature check at all — the
ancestor walk only looked at each ancestor's own_methods, which never
contains a name that only exists through an alias.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
trait T { public function greet(string $s): void {} }
class Base { use T { greet as sayHello; } }
class Child extends Base {
    public function sayHello(int $s): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Child::sayhello() signature mismatch: parameter $s type 'int' is incompatible with parent type 'string'
}
===expect===
