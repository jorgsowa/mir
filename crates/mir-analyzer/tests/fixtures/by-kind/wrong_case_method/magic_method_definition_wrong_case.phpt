===description===
Magic method defined with wrong casing is reported.
===config===
suppress=UnusedParam
===file===
<?php
class Foo {
    public function __CONSTRUCT() {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ WrongCaseMethod: Method name 'Foo::__CONSTRUCT' has incorrect casing; use '__construct'
    public function __Destruct() {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ WrongCaseMethod: Method name 'Foo::__Destruct' has incorrect casing; use '__destruct'
    public function __ToString(): string { return "x"; }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ WrongCaseMethod: Method name 'Foo::__ToString' has incorrect casing; use '__toString'
    public function __CallStatic(string $name, array $args): mixed { return null; }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ WrongCaseMethod: Method name 'Foo::__CallStatic' has incorrect casing; use '__callStatic'
    public function __debuginfo(): array { return []; }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ WrongCaseMethod: Method name 'Foo::__debuginfo' has incorrect casing; use '__debugInfo'
}
===expect===
