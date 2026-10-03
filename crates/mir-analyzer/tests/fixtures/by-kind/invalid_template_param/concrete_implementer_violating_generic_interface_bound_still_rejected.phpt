===description===
Sanity check: fixing the false positive for a concrete class that SATISFIES
`Collection<int>` must not disable rejecting one that doesn't.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'T' inferred as 'StringCollection' does not satisfy bound 'Collection<int>'
===expect===
