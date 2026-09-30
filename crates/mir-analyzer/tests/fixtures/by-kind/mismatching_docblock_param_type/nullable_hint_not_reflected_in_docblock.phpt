===description===
A nullable native hint (`?T`) whose docblock `@param` type doesn't mention
null is itself a contradiction — the hint (PHP's enforced ground truth)
allows null, but the docblock promises a value that never is. This is the
mirror image of `docblock_param_contradicts_hint.phpt`, which only catches
a docblock claiming something the hint disallows.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param string $name
 */
function greet(?string $name): void {}
//                     ^^^^^ MismatchingDocblockParamType: Docblock type 'string' for $name does not match inferred 'string|null'

/**
 * @param list<int> $items
 */
function takesItems(?array $items): void {}
//                         ^^^^^^ MismatchingDocblockParamType: Docblock type 'list<int>' for $items does not match inferred 'array|null'
===expect===
