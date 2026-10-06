===description===
`use A { missingMethod as alias; }` — an unqualified alias naming no method
any used trait declares is a PHP fatal error at class-declaration time.
===file===
<?php
trait A {
    public function foo(): void {}
}

class C {
    use A {
        missingMethod as aliasName;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedTraitAliasMethod: An alias was defined for missingmethod but this method does not exist
    }
}
===expect===
