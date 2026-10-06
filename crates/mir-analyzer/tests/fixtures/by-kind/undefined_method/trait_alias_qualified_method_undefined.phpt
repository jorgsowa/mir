===description===
`use A { A::missingMethod as alias; }` — aliasing a method that doesn't
exist on the named trait is a PHP fatal error at class-declaration time.
===file===
<?php
trait A {
    public function foo(): void {}
}

class C {
    use A {
        A::missingMethod as aliasName;
    }
}
===expect===
UndefinedTraitAliasMethod@8:8-8:38: An alias was defined for A::missingmethod but this method does not exist
