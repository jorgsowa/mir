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

/**
 * @param list<int> $items
 */
function takesItems(?array $items): void {}
===expect===
MismatchingDocblockParamType@5:23-5:28: Docblock type 'string' for $name does not match inferred 'string|null'
MismatchingDocblockParamType@10:27-10:33: Docblock type 'list<int>' for $items does not match inferred 'array|null'
