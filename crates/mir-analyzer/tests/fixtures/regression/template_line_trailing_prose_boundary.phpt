===description===
`parse_template_line`'s bound-token loop didn't stop at trailing same-line
prose — `@template T of ArrayAccess the container type argument` folded
the whole description into the bound string, which then failed to parse
as a real bound and silently lost the constraint entirely. Now stops
once the bound's own brackets are balanced and the next token isn't a
dangling union/intersection continuation.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @template T of ArrayAccess the container type argument
 * @param T $x
 */
function f($x): void {}

f(5);
===expect===
InvalidTemplateParam@8:0-8:4: Template type 'T' inferred as '5' does not satisfy bound 'ArrayAccess'
