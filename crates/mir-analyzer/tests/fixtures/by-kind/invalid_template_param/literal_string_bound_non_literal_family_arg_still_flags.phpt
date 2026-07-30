===description===
Negative control for the `literal-int`/`literal-string` keyword fix: an
argument from a genuinely different type family still violates the bound
— the fix only recognizes the keyword, it doesn't loosen the bound check
itself.
===config===
suppress=UnusedParam
===file===
<?php
class Wrong {}
/**
 * @template A of literal-int|literal-string
 */
final class Box {
    /** @param A $v */
    public function __construct(int|string|Wrong $v) {}
}
new Box(new Wrong());
===expect===
InvalidTemplateParam@10:0-10:20: Template type 'A' inferred as 'Wrong' does not satisfy bound 'int|string'
