===description===
`parent::method()` checks `@template T of static` against the caller's own
late-static-bound receiver, not the parent class `parent::` resolves the
method through — those differ whenever the call happens inside a subclass.
===config===
suppress=UnusedParam
===file===
<?php
abstract class Base {
    /**
     * @template T of static
     * @param T $x
     */
    public static function accept($x): void {}
}
class Mid extends Base {
    public static function relay(Base $x): void {
        parent::accept($x);
    }
    public static function relayValid(): void {
        parent::accept(new Mid());
    }
}
===expect===
InvalidTemplateParam@11:8-11:26: Template type 'T' inferred as 'Base' does not satisfy bound 'Mid'
