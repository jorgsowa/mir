===description===
Sanity check: fixing the false positive for a concrete class that SATISFIES
`Collection<int>` must not disable rejecting one that doesn't.
===config===
suppress=UnusedParam
===file===
<?php
/** @template T */
interface Collection {}
/** @implements Collection<string> */
class StringCollection implements Collection {}

/**
 * @template T of Collection<int>
 * @param T $c
 */
function takesIntCollection($c): void {}

takesIntCollection(new StringCollection());
===expect===
InvalidTemplateParam@13:0-13:42: Template type 'T' inferred as 'StringCollection' does not satisfy bound 'Collection<int>'
