===description===
An int for a `@method` `string` parameter under strict_types is a coercion, as in a non-strict file.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/** @method void rename(string $name) */
class Tag {
    public function __call(string $name, array $args): mixed { return null; }
}

function run(Tag $tag): void {
    $tag->rename('a');
    $tag->rename(5);
//               ^ ArgumentTypeCoercion: Argument $name of rename() expects 'string', got '5' — coercion may fail at runtime
}
