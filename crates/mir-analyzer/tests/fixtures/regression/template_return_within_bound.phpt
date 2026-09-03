===description===
G1: returning the template parameter itself (a param typed `T`, erased to its bound in
the body) or a subtype of the bound from an `@return T` method is compatible. Template
params erase to their bound for the return-site check, so neither case is a false positive.
===config===
suppress=UnusedParam
===file===
<?php
class Animal {}
class Dog extends Animal {}

/**
 * @template T of Animal
 */
class Crate {
    /**
     * @param T $value
     * @return T
     */
    public function identity($value) {
        return $value;
    }
    /** @return T */
    public function makeDog() {
        return new Dog();
    }
}
===expect===
