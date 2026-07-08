===description===
alias renames an abstract (unimplemented) trait method — still reported
===config===
suppress=UnimplementedAbstractMethod
===file===
<?php
trait A {
    abstract public function hello(int $x): string;
}
class C {
    use A {
        A::hello as helloAlias;
    }
}
interface Greets {
    public function helloAlias(int $x): string;
}
class D extends C implements Greets {}
===expect===
UnimplementedInterfaceMethod@13:0-13:38: Class D must implement Greets::helloAlias() from interface
