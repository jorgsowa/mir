===description===
Magic method defined with wrong casing is reported.
===config===
suppress=UnusedParam
===file===
<?php
class Foo {
    public function __CONSTRUCT() {}
    public function __Destruct() {}
    public function __ToString(): string { return "x"; }
    public function __CallStatic(string $name, array $args): mixed { return null; }
    public function __debuginfo(): array { return []; }
}
===expect===
WrongCaseMethod@3:4-3:36: Method name 'Foo::__CONSTRUCT' has incorrect casing; use '__construct'
WrongCaseMethod@4:4-4:35: Method name 'Foo::__Destruct' has incorrect casing; use '__destruct'
WrongCaseMethod@5:4-5:56: Method name 'Foo::__ToString' has incorrect casing; use '__toString'
WrongCaseMethod@6:4-6:83: Method name 'Foo::__CallStatic' has incorrect casing; use '__callStatic'
WrongCaseMethod@7:4-7:55: Method name 'Foo::__debuginfo' has incorrect casing; use '__debugInfo'
