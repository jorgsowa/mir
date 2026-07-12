===description===
UndefinedDocblockClass fires when a property's `@var` docblock names a class
that does not exist and the property has no native type hint.
===config===
suppress=MissingPropertyType
===file===
<?php
class Wallet {
    /** @var NonExistentMoneyClass */
    private $money;
}
===expect===
UndefinedDocblockClass@4:4-4:18: Docblock type 'NonExistentMoneyClass' does not exist
