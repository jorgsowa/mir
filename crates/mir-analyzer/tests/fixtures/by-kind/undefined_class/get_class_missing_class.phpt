===description===
Foo::class in a match arm does not emit UndefinedClass — ::class is a compile-time
string constant that does not require the class to be defined.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {}
class B {}

$a = rand(0, 10) ? new A() : new B();

$a = match (get_class($a)) {
//   ^ +2:1 UnhandledMatchCondition: Unhandled match condition: possibly-unmatched value of type 'string'
    C::class => 5,
};
