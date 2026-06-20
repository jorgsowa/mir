===description===
A child override widening a mixed object+scalar union return type (string|Cat -> string|Animal)
violates return covariance. Now that named_object_return_compatible splits mixed unions per
atom (G5), the override check catches it instead of skipping.
===config===
suppress=UnusedParam
===file===
<?php
class Animal {}
class Cat extends Animal {}
class Base {
    public function make(): string|Cat { return new Cat(); }
}
class Sub extends Base {
    public function make(): string|Animal { return new Animal(); }
}
===expect===
MethodSignatureMismatch@8:4-8:66: Method Sub::make() signature mismatch: return type 'string|Animal' is not a subtype of parent 'string|Cat'
