===description===
A @template T of <alias> bound skipped alias expansion entirely, treating the
alias name as a literal nonexistent class. This fired UndefinedDocblockClass
on the class docblock AND a bogus InvalidTemplateParam on a call that
actually satisfies the (correctly expanded) bound.
===config===
suppress=UnusedParam,MissingPropertyType
===file===
<?php
class Base {}
class Related extends Base {}
class Unrelated {}

/**
 * @psalm-type BaseAlias = Base
 * @template T of BaseAlias
 */
class Box {
    /** @var T */
    private $item;
    /** @param T $item */
    public function __construct($item) {
        $this->item = $item;
    }
}

new Box(new Related());
new Box(new Unrelated());
===expect===
InvalidTemplateParam@20:0-20:24: Template type 'T' inferred as 'Unrelated' does not satisfy bound 'Base'
