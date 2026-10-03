===description===
UndefinedDocblockClass fires when a property's `@var` docblock names a class
that does not exist and the property has no native type hint.
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Wallet {
    /** @var NonExistentMoneyClass */
    private $money;
//  ^^^^^^^^^^^^^^ UndefinedDocblockClass: Docblock type 'NonExistentMoneyClass' does not exist
}
===expect===
