===description===
With several templates `template-type` picks the one that is named, also when the object is itself a template bound to the class.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/**
 * @template K
 * @template V
 */
final class Entry {
    /**
     * @param K $key
     * @param V $value
     */
    public function __construct(public mixed $key, public mixed $value) {}
}

/**
 * @template TEntry of Entry
 * @param TEntry $entry
 * @return template-type<TEntry, Entry, 'K'>
 */
function keyOf(Entry $entry): mixed {
    return $entry->key;
}

/**
 * @template TEntry of Entry
 * @param TEntry $entry
 * @return template-type<TEntry, Entry, 'V'>
 */
function valueOf(Entry $entry): mixed {
    return $entry->value;
}

function takesInt(int $value): void {}

$entry = new Entry(7, 'seven');
takesInt(keyOf($entry));
takesInt(valueOf($entry));
//       ^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of takesInt() expects 'int', got 'string'
