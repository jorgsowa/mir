===description===
A final class named only in a property's `@var` docblock tag (no native
property type naming it) must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Money {}

class Wallet {
    /** @var Money */
    private $money;

    public function balance() {
        return $this->money;
    }
}

(new Wallet())->balance();
===expect===
MissingPropertyType@6:4-6:18: Property Wallet::$money has no type annotation
