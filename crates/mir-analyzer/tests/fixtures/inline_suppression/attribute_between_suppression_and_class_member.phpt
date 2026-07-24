===description===
Same bug as the top-level counterpart, but for a class member: a PHP 8
attribute between a suppression comment and a method declaration defeated
the suppression, since next_code_line stopped on the attribute line
instead of continuing to the method itself.
===config===
suppress=UndefinedAttributeClass,UnusedParam
===file===
<?php
class Foo {
    /** @mir-ignore UndefinedClass */
    #[Bar]
    public function useUndefined(UndefinedTypeX $x): void {
    }
}
===expect===
