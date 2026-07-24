===description===
A PHP 8 attribute between a suppression comment and the top-level
declaration it targets defeated the suppression entirely --
next_code_line's skip-comments loop treats `#[` as code (correctly, so it
doesn't misparse an attribute as a plain comment) but then stopped right
there instead of continuing past it, landing the suppression target on
the attribute line while the declaration's own diagnostic fires one line
further down.
===config===
suppress=UndefinedAttributeClass,UnusedParam
===file===
<?php
/** @mir-ignore UndefinedClass */
#[Foo]
function useUndefined(UndefinedTypeX $x): void {
}
===expect===
